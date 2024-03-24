use std::{
    fmt::{format, Display},
    num::{ParseFloatError, ParseIntError},
};

use thiserror::Error;

use crate::{
    objects::{Entity, Number, NumberType, Position},
    statements::FileInfo,
};

#[derive(Debug)]
pub struct CompileError {
    file_path: String,
    line: u32,
    column: u32,
    error_type: CompileErrorType,
}
impl CompileError {
    pub fn new(file_info: &FileInfo, position: Position, error_type: CompileErrorType) -> Self {
        Self {
            file_path: file_info.path.clone(),
            line: position.line as u32,
            column: position.column as u32,
            error_type,
        }
    }
}
impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Compilation Error: \n  File: {}\n  Line: {}, Column: {}\n  Error: {}",
            self.file_path, self.line, self.column, self.error_type
        )
    }
}
impl std::error::Error for CompileError {}

#[derive(Debug)]
pub enum CompileErrorType {
    //#[error("This file is empty.")]
    //FileEmpty,
    //#[error("The path '{0}' does not lead to a valid file.")]
    //InvalidPath(String),
    InvalidKeyword(String),
    LineEmpty(String),
    TooManyCharacters(String, usize, usize),
    KeywordWithoutArguments(String, String),
    InvalidCharacters(String),
    InvalidNumberPrefix(char),
    InvalidInt(String, ParseIntError),
    StringSectionEmpty(String),
    IncorrectArgumentCount(String, usize, usize),
    IncorrectNumberType(Number, NumberType),
    DuplicateNumberType(NumberType),
    IncorrectSeparator(String, char),
    InvalidCoordinate(String, ParseFloatError),
    InvalidEntityType(String),
}
impl Display for CompileErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeyword(keyword) => {
                write!(f, "Keyword '{keyword}' is invalid.")
            }
            Self::LineEmpty(line) => {
                write!(f, "Line '{line}' is empty.")
            }
            Self::TooManyCharacters(statement, expected, found) => {
                write!(
                    f,
                    "Too many characters in '{statement}': Expected '{expected}', found '{found}'."
                )
            }
            Self::KeywordWithoutArguments(keyword, statement) => {
                write!(
                    f,
                    "Statement '{statement}' specifies keyword '{keyword}' without arguments."
                )
            }
            Self::InvalidCharacters(statement) => {
                write!(f, "Statement '{statement}' contains invalid characters.")
            }
            Self::InvalidNumberPrefix(char) => {
                write!(f, "Character '{char}' is an invalid number prefix.")
            }
            Self::InvalidInt(number, error) => {
                write!(f, "Number '{number}' is not a valid int: {error}")
            }
            Self::StringSectionEmpty(string) => {
                write!(f, "Section of string '{string}' is empty.")
            }
            Self::IncorrectArgumentCount(statement, expected, found) => {
                write!(
                    f,
                    "Incorrect number of arguments in '{statement}': Expected '{expected}', found '{found}'."
                )
            }
            Self::IncorrectNumberType(number, expected) => {
                write!(
                    f,
                    "Number '{number}' is of incorrect type. Expected: '{expected}'."
                )
            }
            Self::DuplicateNumberType(number_type) => {
                write!(
                    f,
                    "Tried creating NumberSet with duplicate number type: {number_type}"
                )
            }
            Self::IncorrectSeparator(statement, expected) => {
                write!(
                    f,
                    "Statement '{statement}' had the incorrect separator: Expected '{expected}'."
                )
            }
            Self::InvalidCoordinate(coordinate, error) => {
                write!(f, "Coordinate '{coordinate}' is invalid: {error}")
            }
            Self::InvalidEntityType(argument) => {
                write!(
                    f,
                    "Entity type '{argument}' is invalid. Expected one of: [{}]",
                    Entity::TYPES.map(|s| format!("\"{s}\"")).join(", ")
                )
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum GenericError {
    #[error("The path '{0}' does not lead to a valid file.")]
    InvalidPath(String),
    #[error("Block queue is empty.")]
    BlockQueueEmpty,
    #[error("Pattern '{0}' is not a valid regex: {1}")]
    InvalidRegex(&'static str, #[source] regex::Error),
    #[error("The file with path '{0}' does not exist.")]
    FileNotExist(String),
    #[error("Could to compile one or more files due to errors:\n{0}")]
    Collection(String),
}
