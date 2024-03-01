use std::{fmt::Display, str::FromStr};

use crate::{
    errors::{CompileError, CompileErrorType as ErrorType, GenericError},
    objects::{self, Entity, Position, Rotation, Scale, TrackedChar, Translation},
};

use anyhow::{bail, ensure, Result as AResult};
use regex::Regex;

pub struct FileInfo {
    pub path: String,
    pub eof: TrackedChar,
}
impl FileInfo {
    pub fn new(path: String, eof: TrackedChar) -> Self {
        Self { path, eof }
    }
}

pub struct Program {
    base_block: Statement,
}
impl Program {
    pub fn parse_from_file<I: Iterator<Item = TrackedChar>>(
        file_info: FileInfo,
        contents: &mut I,
    ) -> AResult<Self> {
        let base_block = Statement::Block(Block::new(NumberSet::default(), Vec::new()));

        Ok(Program {
            base_block: Self::parse_block(file_info, contents, base_block)?,
        })
    }
    fn parse_block<I: Iterator<Item = TrackedChar>>(
        file_info: FileInfo,
        contents: &mut I,
        mut base_block: Statement,
    ) -> AResult<Statement> {
        let Statement::Block(ref mut current_block) = base_block else {
            bail!(GenericError::BlockNotBlock(base_block.clone()))
        };
        let next_statement =
            Statement::parse_from_file(file_info, contents, current_block.numbers)?;
        match next_statement {
            Statement::Block(_) => {
                current_block.statements.push(Self::parse_block(
                    file_info,
                    contents,
                    next_statement,
                )?);
            }
            Statement::BlockEnd => {}
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
    BlockEnd,
    Keyword(NumberSet, Entity, KeywordStatement),
    End(u32),
}
impl Statement {
    const OBJECT_KEYWORD: &'static str = "object";
    const ANIMATION_KEYWORD: &'static str = "anim";

    const COMMENT_REGEX: &'static str = r"#.*?\n";
    const NAME_REGEX: &'static str = r"^[A-Za-z0-9_\-]*$";
    fn parse_from_file<I: Iterator<Item = TrackedChar>>(
        file_info: FileInfo,
        contents: &mut I,
        current_numbers: NumberSet,
    ) -> AResult<Self> {
        let comment_regex: Result<Regex, GenericError> = Regex::new(Self::COMMENT_REGEX)
            .map_err(|err| GenericError::InvalidRegex(Self::COMMENT_REGEX, err));

        let raw_buffer: (String, Position) = contents
            .take_while(|c| ![';', '{', '}'].contains(&c.character))
            .fold((String::new(), Position::default()), |mut acc, c| {
                if acc.0.is_empty() {
                    acc.1 = c.position;
                }
                acc.0.push(c.character);
                acc
            });
        let buffer_string: std::borrow::Cow<'_, str> =
            comment_regex?.replace_all(&raw_buffer.0, "");
        let buffer_string: &str = buffer_string.trim();
        let buffer: (&str, Position) = (buffer_string, raw_buffer.1);

        match &buffer {
            _ if buffer.0.starts_with("}") => return Self::parse_block_end(file_info, buffer),
            _ if buffer.0.starts_with(Self::OBJECT_KEYWORD) => {
                return Self::parse_object(file_info, buffer)
            }
            _ if buffer.0.starts_with(Self::ANIMATION_KEYWORD) => {
                return Self::parse_animation(file_info, buffer)
            }
            _ => {}
        }
        let mut words = buffer.0.split(' ');

        let first_word = words.next().ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::LineEmpty(buffer.0.to_string()),
        ))?;

        if Number::is_number(first_word) {}

        todo!()
    }

    fn parse_block_end(file_info: FileInfo, buffer: (&str, Position)) -> AResult<Statement> {
        assert!(buffer.0.starts_with("}"));
        ensure!(
            buffer.0.len() > 1,
            CompileError::new(
                file_info,
                buffer.1 + 1,
                ErrorType::TooManyCharacters(buffer.0.to_owned(), 1, buffer.0.len())
            )
        );
        Ok(Self::BlockEnd)
    }
    fn parse_object(file_info: FileInfo, buffer: (&str, Position)) -> AResult<Statement> {
        assert!(buffer.0.starts_with(Self::OBJECT_KEYWORD));
        let (keyword, argument) = buffer.0.split_once(' ').ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::KeywordWithoutArguments(
                Self::OBJECT_KEYWORD.to_string(),
                buffer.0.to_string(),
            ),
        ))?;
        todo!()
    }
    fn parse_animation(file_info: FileInfo, buffer: (&str, Position)) -> AResult<Statement> {
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
    type Err = ErrorType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "translate" | "move" | "m" => Ok(Self::Translate),
            "rotate" | "turn" | "r" => Ok(Self::Rotate),
            "scale" | "size" | "s" => Ok(Self::Scale),
            //"spawn" => Ok(Self::Spawn),
            _ => Err(ErrorType::InvalidKeyword(s.to_string())),
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

#[derive(Debug, Clone, Copy)]
struct Number {
    value: u32,
    number_type: NumberType,
}
impl Number {
    fn is_number(value: &str) -> bool {
        value.starts_with(NumberType::DELAY_PREFIX)
            || value.starts_with(NumberType::DURATION_PREFIX)
    }
}

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

#[derive(Debug, Clone, Copy)]
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
