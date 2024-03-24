use std::{
    fmt::Display,
    ops::{Add, Sub},
    str::FromStr,
};

use quaternion_core::{RotationSequence, RotationType};
use regex::Regex;

use crate::{
    errors::{CompileErrorType as ErrorType, GenericError},
    statements::KeywordStatement,
};

pub trait SimpleTransformation {
    fn to_statement(self) -> KeywordStatement;

    fn get_x(transformation: Transformation) -> f32;
    fn get_y(transformation: Transformation) -> f32;
    fn get_z(transformation: Transformation) -> f32;

    fn transform(&self, transformation: Transformation) -> Transformation;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Translation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Translation {
    pub fn compile(&self) -> String {
        format!("translation: [{}f,{}f,{}f]", self.x, self.y, self.z)
    }
}
impl From<(f32, f32, f32)> for Translation {
    fn from(value: (f32, f32, f32)) -> Self {
        Translation {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}
impl SimpleTransformation for Translation {
    fn to_statement(self) -> KeywordStatement {
        KeywordStatement::Translate(self)
    }

    fn get_x(transformation: Transformation) -> f32 {
        transformation.translation.x
    }
    fn get_y(transformation: Transformation) -> f32 {
        transformation.translation.y
    }
    fn get_z(transformation: Transformation) -> f32 {
        transformation.translation.z
    }

    fn transform(&self, transformation: Transformation) -> Transformation {
        transformation.with_translation(*self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}
impl Rotation {
    fn to_radians(degrees: f32) -> f32 {
        degrees * std::f32::consts::PI / 180.0
    }

    pub fn compile(&self) -> String {
        let quaternion = quaternion_core::from_euler_angles(
            RotationType::Intrinsic,
            RotationSequence::YZX,
            [
                Self::to_radians(self.pitch),
                Self::to_radians(self.yaw),
                Self::to_radians(self.roll),
            ],
        );
        format!(
            "left_rotation: [{}f,{}f,{}f,{}f]",
            quaternion.0, quaternion.1[0], quaternion.1[1], quaternion.1[2],
        )
    }
}
impl From<(f32, f32, f32)> for Rotation {
    fn from(value: (f32, f32, f32)) -> Self {
        Rotation {
            yaw: value.0,
            pitch: value.1,
            roll: value.2,
        }
    }
}
impl SimpleTransformation for Rotation {
    fn to_statement(self) -> KeywordStatement {
        KeywordStatement::Rotate(self)
    }

    fn get_x(transformation: Transformation) -> f32 {
        transformation.rotation.yaw
    }
    fn get_y(transformation: Transformation) -> f32 {
        transformation.rotation.pitch
    }
    fn get_z(transformation: Transformation) -> f32 {
        transformation.rotation.roll
    }

    fn transform(&self, transformation: Transformation) -> Transformation {
        transformation.with_rotation(*self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Scale {
    pub fn compile(&self) -> String {
        format!("scale: [{}f,{}f,{}f]", self.x, self.y, self.z)
    }
}
impl From<(f32, f32, f32)> for Scale {
    fn from(value: (f32, f32, f32)) -> Self {
        Scale {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}
impl SimpleTransformation for Scale {
    fn to_statement(self) -> KeywordStatement {
        KeywordStatement::Scale(self)
    }
    fn get_x(transformation: Transformation) -> f32 {
        transformation.scale.x
    }
    fn get_y(transformation: Transformation) -> f32 {
        transformation.scale.y
    }
    fn get_z(transformation: Transformation) -> f32 {
        transformation.scale.z
    }
    fn transform(&self, transformation: Transformation) -> Transformation {
        transformation.with_scale(*self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Transformation {
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
}
impl Transformation {
    pub fn with_translation(&self, translation: Translation) -> Self {
        Transformation {
            translation,
            ..*self
        }
    }
    pub fn with_rotation(&self, rotation: Rotation) -> Self {
        Transformation { rotation, ..*self }
    }
    pub fn with_scale(&self, scale: Scale) -> Self {
        Transformation { scale, ..*self }
    }
}

pub enum EntityType {}
impl EntityType {
    pub const TYPES: [&'static str; 3] = ["block_display", "item_display", "text_display"];
}

#[derive(Debug, Clone, Copy)]
pub struct TrackedChar {
    pub position: Position,
    pub character: char,
}
impl TrackedChar {
    pub fn new(line: usize, column: usize, character: char) -> Self {
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
    fn new(line: usize, column: usize) -> Self {
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
    pub comment: Regex,
    pub valid_character: Regex,
}
impl Regexes {
    const COMMENT_REGEX: &'static str = r"#.*?\n";
    const VALID_CHARACTER_REGEX: &'static str = r"^[A-Za-z0-9_\-]*$";

    pub fn new() -> anyhow::Result<Self> {
        Ok(Regexes {
            comment: Regex::new(Self::COMMENT_REGEX)
                .map_err(|err| GenericError::InvalidRegex(Self::COMMENT_REGEX, err))?,
            valid_character: Regex::new(Self::VALID_CHARACTER_REGEX)
                .map_err(|err| GenericError::InvalidRegex(Self::VALID_CHARACTER_REGEX, err))?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Number {
    pub value: u32,
    pub number_type: NumberType,
}
impl FromStr for Number {
    type Err = ErrorType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix_str, number_str) = s.split_at(1);
        let number_type: NumberType = prefix_str
            .chars()
            .next()
            .ok_or(ErrorType::StringSectionEmpty(s.to_string()))?
            .try_into()?;
        let value: u32 = number_str
            .parse()
            .map_err(|err| ErrorType::InvalidInt(number_str.to_string(), err))?;
        Ok(Self { value, number_type })
    }
}
impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.value, self.number_type)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NumberSet {
    pub delay: u32,
    pub duration: u32,
}
impl NumberSet {
    pub fn new(delay: u32, duration: u32) -> Self {
        Self { delay, duration }
    }

    pub fn new_unordered(value_1: Number, value_2: Number) -> Result<Self, ErrorType> {
        match (value_1.number_type, value_2.number_type) {
            (NumberType::Delay, NumberType::Duration) => Ok(Self {
                delay: value_1.value,
                duration: value_2.value,
            }),
            (NumberType::Duration, NumberType::Delay) => Ok(Self {
                delay: value_2.value,
                duration: value_1.value,
            }),
            _ => Err(ErrorType::DuplicateNumberType(value_1.number_type)),
        }
    }

    pub fn new_with_default(number: Number, default: NumberSet) -> Self {
        match number.number_type {
            NumberType::Delay => Self::new(number.value, default.duration),
            NumberType::Duration => Self::new(default.delay, number.value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType {
    Delay,
    Duration,
}
impl NumberType {
    const DELAY_PREFIX: char = '@';
    const DURATION_PREFIX: char = '%';

    pub fn has_prefix(string: &str) -> bool {
        string.starts_with(Self::DELAY_PREFIX) || string.starts_with(Self::DURATION_PREFIX)
    }
}
impl Display for NumberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberType::Delay => write!(f, "Delay: ({})", Self::DELAY_PREFIX),
            NumberType::Duration => write!(f, "Duration: ({})", Self::DURATION_PREFIX),
        }
    }
}
impl TryFrom<char> for NumberType {
    type Error = ErrorType;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            Self::DELAY_PREFIX => Ok(Self::Delay),
            Self::DURATION_PREFIX => Ok(Self::Duration),
            _ => Err(ErrorType::InvalidNumberPrefix(value)),
        }
    }
}
