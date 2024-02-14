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
    #[error("Failed to parse block definition '{0}': {1}")]
    InvalidBlockDefinition(String, #[source] NumberSetError),
    #[error("The statement '{0}' has too few elements: {1}/{2}.")]
    TooFewElements(String, u8, u8),
    #[error("Failed to parse coordinate: {0}.")]
    InvalidCoordinate(String, #[source] std::num::ParseFloatError),
    #[error("One or more item in the collection threw an error:\n {0}.")]
    InvalidCollection(String),
    #[error("Number '{0}' did not contain a discriminator.")]
    InvalidDiscriminator(String),
    #[error("Number '{0}' is not a valid number: {1}")]
    InvalidNumber(String, #[source] std::num::ParseIntError),
}
#[derive(Debug, Error)]
pub enum NumberSetError {
    #[error("The numbers are both of type {0}.")]
    Duplicate(crate::file_reader::NumberType),
    #[error("Too many numbers: {0}/2")]
    TooMany(u32),
    #[error("Too few numbers: {0}/2")]
    TooFew(u32),
}
