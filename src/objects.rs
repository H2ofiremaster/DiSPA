use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use regex::Regex;

use crate::errors::{CompileErrorType as ErrorType, GenericError};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Translation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Translation {
    pub const fn new(coordinates: (f32, f32, f32)) -> Self {
        Self {
            x: coordinates.0,
            y: coordinates.1,
            z: coordinates.2,
        }
    }
    pub fn compile(&self) -> String {
        format!("translation: [{}f,{}f,{}f]", self.x, self.y, self.z)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rotation {
    pub axis: [f32; 3],
    pub angle: f32,
}
impl Rotation {
    pub const fn new(axis: [f32; 3], angle: f32) -> Self {
        Self { axis, angle }
    }
    pub fn compile(&self) -> String {
        let quaternion = quaternion_core::from_axis_angle(self.axis, self.angle.to_radians());
        format!(
            "left_rotation: [{}f,{}f,{}f,{}f]",
            quaternion.1[0], quaternion.1[1], quaternion.1[2], quaternion.0,
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Scale {
    pub const fn new(coordinates: (f32, f32, f32)) -> Self {
        Self {
            x: coordinates.0,
            y: coordinates.1,
            z: coordinates.2,
        }
    }
    pub fn compile(&self) -> String {
        format!("scale: [{}f,{}f,{}f]", self.x, self.y, self.z)
    }
}
// #[derive(Debug, Default, Clone, Copy)]
// pub struct Transformation {
//     pub translation: Translation,
//     pub rotation: Rotation,
//     pub scale: Scale,
// }
// impl Transformation {
//     pub const fn with_translation(&self, translation: Translation) -> Self {
//         Self {
//             translation,
//             ..*self
//         }
//     }
//     pub const fn with_rotation(&self, rotation: Rotation) -> Self {
//         Self { rotation, ..*self }
//     }
//     pub const fn with_scale(&self, scale: Scale) -> Self {
//         Self { scale, ..*self }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity(String);
impl Entity {
    pub const TYPES: [&'static str; 3] = ["block_display", "item_display", "text_display"];

    pub fn new(string: String, validator: &Regex) -> Result<Self, ErrorType> {
        if validator.is_match(&string) {
            Ok(Self(string))
        } else {
            Err(ErrorType::InvalidEntityName(string))
        }
    }
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrackedChar {
    pub position: Position,
    pub character: char,
}
impl TrackedChar {
    pub const fn new(line: usize, column: usize, character: char) -> Self {
        Self {
            position: Position::new(line, column),
            character,
        }
    }
}
impl Display for TrackedChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}': {}", self.character, self.position)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
impl Position {
    const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}
impl Add<usize> for Position {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            line: self.line,
            column: self.column + rhs,
        }
    }
}
impl Add<(usize, usize)> for Position {
    type Output = Self;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        Self {
            line: self.line + rhs.0,
            column: self.column + rhs.1,
        }
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
impl Sub<usize> for Position {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self {
            line: self.line,
            column: self.column - rhs,
        }
    }
}

pub struct Regexes {
    pub name: Regex,
}
impl Regexes {
    const NAME: &'static str = r"^[A-Za-z0-9_\-]*$";

    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            name: Regex::new(Self::NAME)
                .map_err(|err| GenericError::InvalidRegex(Self::NAME, err))?,
        })
    }
}
