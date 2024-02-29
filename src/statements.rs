use std::{fmt::Display, str::FromStr};

use crate::{
    errors::{CompileErrorType, GenericError},
    objects::{Entity, Rotation, Scale, Translation},
};

use anyhow::{bail, ensure, Result as AResult};

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
    base_block: Statement,
}
impl Program {
    pub fn parse_from_file(
        file_path: &str,
        contents: &mut impl Iterator<Item = TrackedChar>,
    ) -> AResult<Self> {
        let mut base_block = Statement::Block(Block::new(NumberSet::default(), Vec::new()));

        Ok(Program {
            base_block: Self::parse_block(file_path, contents, base_block)?,
        })
    }
    fn parse_block(
        file_path: &str,
        contents: &mut impl Iterator<Item = TrackedChar>,
        mut base_block: Statement,
    ) -> AResult<Statement> {
        let Statement::Block(ref mut current_block) = base_block else {
            bail!(GenericError::BlockNotBlock(base_block.clone()))
        };
        let next_statement =
            Statement::parse_from_file(file_path, contents, &current_block.numbers)?;
        match next_statement {
            Statement::Block(_) => {
                current_block.statements.push(Self::parse_block(
                    file_path,
                    contents,
                    next_statement,
                )?);
            }
            Statement::Empty => {}
            _ => {
                current_block.statements.push(next_statement);
            }
        };
        Ok(base_block)
    }
}

#[derive(Debug, Clone)]
struct Block {
    numbers: NumberSet,
    statements: Vec<Statement>,
}
impl Block {
    fn new(numbers: NumberSet, statements: Vec<Statement>) -> Self {
        Self {
            numbers,
            statements,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    ObjectName(String),
    AnimationName(String),
    Block(Block),
    Keyword(NumberSet, Entity, KeywordStatement),
    End(u32),
    Empty,
}
impl Statement {
    pub fn parse_from_file(
        file_path: &str,
        contents: &mut impl Iterator<Item = TrackedChar>,
        current_numbers: &NumberSet,
    ) -> AResult<Self> {
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

#[derive(Debug, Clone)]
enum KeywordStatement {
    Translate(Translation),
    Rotate(Rotation),
    Scale(Scale),
    //Spawn(Entity, EntityType),
}

enum EntityType {}

#[derive(Debug, Default, Clone, Copy)]
struct NumberSet {
    delay: u32,
    duration: u32,
}
impl NumberSet {
    fn new(delay: u32, duration: u32) -> Self {
        Self { delay, duration }
    }
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
