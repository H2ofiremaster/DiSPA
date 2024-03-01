use std::fmt::Display;

use thiserror::Error;

use crate::{
    objects::Position,
    statements::{FileInfo, Statement},
};

#[derive(Debug)]
pub struct CompileError {
    file_path: String,
    line: u32,
    column: u32,
    error_type: CompileErrorType,
}
impl CompileError {
    pub fn new(file_info: FileInfo, position: Position, error_type: CompileErrorType) -> Self {
        let file_path = file_info.path;
        let line = position.line as u32;
        let column = position.column as u32;
        Self {
            file_path,
            line,
            column,
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
    UnexpectedEof(String),
    LineEmpty(String),
    TooManyCharacters(String, usize, usize),
    KeywordWithoutArguments(String, String),
}
impl Display for CompileErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeyword(keyword) => {
                write!(f, "Keyword '{keyword}' is invalid.")
            }
            Self::UnexpectedEof(expected) => {
                write!(f, "Unexpected end of file: Expected {expected}.")
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
        }
    }
}

#[derive(Debug, Error)]
pub enum NumberSetError {
    #[error("The numbers are both of type {0}.")]
    Duplicate(crate::statements::NumberType),
    #[error("Too many numbers: {0}/2")]
    TooMany(u32),
    #[error("Too few numbers: {0}/2")]
    TooFew(u32),
}

#[derive(Debug, Error)]
pub enum GenericError {
    #[error("The path '{0}' does not lead to a valid file.")]
    InvalidPath(String),
    #[error("Block queue is empty.")]
    BlockQueueEmpty,
    #[error("Block '{0:?}' is not of type 'block'.")]
    BlockNotBlock(Statement),
    #[error("Pattern '{0}' is not a valid regex: {1}")]
    InvalidRegex(&'static str, #[source] regex::Error),
}
