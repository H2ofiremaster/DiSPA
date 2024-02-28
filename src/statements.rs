use std::{fmt::Display, str::FromStr};

use crate::{
    errors::CompileErrorType,
    objects::{Entity, Rotation, Scale, Translation},
};

pub struct TrackedChar {
    line: usize,
    column: usize,
    character: char,
}
impl TrackedChar {
    pub fn new(line: usize, column: usize, character: char) -> Self {
        Self {
            line,
            column,
            character,
        }
    }
}

pub struct Program {
    statements: Statement,
}
impl Program {
    pub fn parse(file_path: &str, mut contents: &mut impl Iterator<Item = TrackedChar>) {
        todo!()
    }
}

enum Statement {
    ObjectName(String),
    AnimationName(String),
    Block(NumberSet, Vec<Statement>),
    Keyword(NumberSet, Entity, KeywordStatement),
    End(u32),
}
impl Statement {
    pub fn parse(file_path: &str, mut contents: &mut impl Iterator<Item = TrackedChar>) {
        todo!()
    }
}

enum Keyword {
    Translate,
    Rotate,
    Scale,
    //Spawn,
}
impl FromStr for Keyword {
    type Err = CompileErrorType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "translate" | "move" | "m" => Ok(Self::Translate),
            "rotate" | "turn" | "r" => Ok(Self::Rotate),
            "scale" | "size" | "s" => Ok(Self::Scale),
            //"spawn" => Ok(Self::Spawn),
            _ => Err(CompileErrorType::InvalidKeyword(s.to_string())),
        }
    }
}

enum KeywordStatement {
    Translate(Translation),
    Rotate(Rotation),
    Scale(Scale),
    //Spawn(Entity, EntityType),
}

enum EntityType {}

#[derive(Debug)]
struct NumberSet {
    delay: u32,
    duration: u32,
}

#[derive(Debug)]
pub enum NumberType {
    Delay,
    Duration,
}
impl NumberType {
    const DELAY_PREFIX: char = '@';
    const DURATION_PREFIX: char = '%';
}
impl Display for NumberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberType::Delay => write!(f, "Delay: ({})", Self::DELAY_PREFIX),
            NumberType::Duration => write!(f, "Duration: ({})", Self::DURATION_PREFIX),
        }
    }
}
