use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

use thiserror::Error;

use crate::{
    objects::{Entity, Position},
    statements::FileInfo,
};

#[derive(Debug)]
pub struct CompileError {
    file_path: String,
    line: usize,
    column: usize,
    error_message: String,
}
#[allow(clippy::needless_pass_by_value)]
impl CompileError {
    pub fn new(file_info: &FileInfo, position: Position, error_type: CompileErrorType) -> Self {
        Self {
            file_path: file_info.path.clone(),
            line: position.line,
            column: position.column,
            error_message: format!("{error_type}"),
        }
    }
}
impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Compilation Error: \n  File: {}\n  Line: {}, Column: {}\n  Error: {}",
            self.file_path, self.line, self.column, self.error_message
        )
    }
}
impl std::error::Error for CompileError {}

#[derive(Debug)]
pub enum CompileErrorType<'a> {
    InvalidKeyword(&'a str),
    LineEmpty(&'a str),
    InvalidCharacters(&'a str),
    InvalidInt(&'a str, ParseIntError),
    InvalidFloat(&'a str, ParseFloatError),
    IncorrectArgumentCount(&'a str, usize, usize),
    InvalidCoordinate(&'a str, ParseFloatError),
    InvalidAxis(&'a str),
    InvalidEntityType(&'a str),
    InvalidEntityName(&'a str),
    InvalidState(&'a str),
    NoAnimationName(&'a str),
}
impl Display for CompileErrorType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeyword(keyword) => {
                write!(f, "Keyword '{keyword}' is invalid.")
            }
            Self::LineEmpty(line) => {
                write!(f, "Line '{line}' is empty.")
            }
            Self::InvalidCharacters(statement) => {
                write!(f, "Statement '{statement}' contains invalid characters.")
            }
            Self::InvalidInt(number, error) => {
                write!(f, "Number '{number}' is not a valid integer: {error}")
            }
            Self::InvalidFloat(number, error) => {
                write!(f, "Number '{number}' is not a valid float: {error}")
            }
            Self::IncorrectArgumentCount(statement, expected, found) => {
                write!(
                    f,
                    "Incorrect number of arguments in '{statement}': Expected '{expected}', found '{found}'."
                )
            }
            Self::InvalidCoordinate(coordinate, error) => {
                write!(f, "Coordinate '{coordinate}' is invalid: {error}")
            }
            Self::InvalidAxis(axis) => {
                write!(f, "'{axis}' is not a valid axis.")
            }
            Self::InvalidEntityType(argument) => {
                write!(
                    f,
                    "Entity type '{argument}' is invalid. Expected one of: [{}]",
                    Entity::TYPES.map(|s| format!("\"{s}\"")).join(", ")
                )
            }
            Self::InvalidEntityName(name) => {
                write!(f, "Entity name '{name}' contains invalid characters.")
            }
            Self::InvalidState(state) => {
                write!(f, "State '{state}' is invalid.")
            }
            Self::NoAnimationName(argument) => {
                write!(
                    f,
                    "Argument '{argument}' containes object name, but no animation name."
                )
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum GenericError {
    #[error("The path '{0}' does not lead to a valid file: ")]
    InvalidPath(String, String),
    #[error("Pattern '{0}' is not a valid regex: {1}")]
    InvalidRegex(&'static str, #[source] regex::Error),
    #[error("The file with path '{0}' does not exist.")]
    FileNotExist(String),
    #[error("Could to compile one or more files due to errors:\n{0}")]
    Collection(String),
}
