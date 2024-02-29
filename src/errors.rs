use std::fmt::Display;

use thiserror::Error;

use crate::statements::Statement;

#[derive(Debug)]
pub struct CompileError {
    file_path: String,
    line: u32,
    column: u32,
    error_type: CompileErrorType,
}
impl CompileError {
    pub fn new(file_path: String, line: u32, column: u32, error_type: CompileErrorType) -> Self {
        CompileError {
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
    //#[error("Pattern '{0}' is not a valid regex.")]
    //InvalidRegex(&'static str),
    InvalidKeyword(String),
}
impl Display for CompileErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeyword(keyword) => {
                write!(f, "Keyword '{keyword}' is invalid.")
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
}
