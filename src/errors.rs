use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("This file is empty.")]
    FileEmpty,
    #[error("Thie path '{0}' does not lead to a valid file.")]
    InvalidPath(String),
    #[error("Pattern '{0}' is not a valid regex.")]
    InvalidRegex(&'static str),
    #[error("Keyword '{0}' specified, but no name was provided.")]
    NotNamed(String),
    #[error("{0} '{1}' contains invalid characters.")]
    InvalidCharacters(&'static str, String),
    #[error("Unbalanced brackets.")]
    UnbalancedBrackets,
    #[error("The statement '{0}' doesn't contain a keyword.")]
    NoKeyword(String),
    #[error("The statement '{0}' contains an invalid keyword.")]
    InvalidKeyword(String),
    #[error("The block queue is empty.")]
    BlockQueueEmpty,
    #[error("The block definition '{0}' does not contain any numbers.")]
    BlockNoNumbers(String),
    #[error("The block definition '{0}' contains two numbers of the same type.")]
    BlockDuplicateNumbers(String),
    #[error("The block definition '{0}' contains too many numbers.")]
    BlockTooManyNumbers(String),
    #[error("The statement '{0}' has too few elements: {1}/{2}")]
    TooFewElements(String, u8, u8),
    #[error("Failed to parse coordinate: {0}")]
    InvalidCoordinate(String, #[source] std::num::ParseFloatError),
    #[error("One or more item in the collection threw an error:\n {0}")]
    InvalidCollection(String),
}
