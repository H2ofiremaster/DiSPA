use std::str::FromStr;

use crate::{
    errors::{CompileError, CompileErrorType as ErrorType, GenericError},
    objects::{
        Entity, Number, NumberSet, NumberType, Position, Regexes, Rotation, Scale, TrackedChar,
        Translation,
    },
};

use anyhow::{bail, ensure, Result as AResult};
use regex::Regex;

#[derive(Debug, Clone)]
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
            base_block: Self::parse_block(file_info, contents, base_block, Regexes::new()?)?,
        })
    }
    fn parse_block<I: Iterator<Item = TrackedChar>>(
        file_info: FileInfo,
        contents: &mut I,
        mut base_block: Statement,
        regexes: Regexes,
    ) -> AResult<Statement> {
        let Statement::Block(ref mut base_block_data) = base_block else {
            bail!(GenericError::BlockNotBlock(base_block.clone()))
        };
        let next_statement = Statement::parse_from_file(
            file_info.clone(),
            contents,
            base_block_data.numbers,
            &regexes,
        )?;
        match next_statement {
            Statement::Block(_) => {
                base_block_data.statements.push(Self::parse_block(
                    file_info,
                    contents,
                    next_statement,
                    regexes,
                )?);
            }
            Statement::BlockEnd => {}
            _ => {
                base_block_data.statements.push(next_statement);
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
    const STATEMENT_END_CHAR: char = ';';
    const BLOCK_START_CHAR: char = '{';
    const BLOCK_END_CHAR: char = '{';

    fn parse_from_file<I: Iterator<Item = TrackedChar>>(
        file_info: FileInfo,
        contents: &mut I,
        current_numbers: NumberSet,
        regexes: &Regexes,
    ) -> AResult<Self> {
        let raw_buffer: (String, Position) = contents
            .take_while(|c| {
                ![
                    Self::STATEMENT_END_CHAR,
                    Self::BLOCK_START_CHAR,
                    Self::BLOCK_END_CHAR,
                ]
                .contains(&c.character)
            })
            .fold((String::new(), Position::default()), |mut acc, c| {
                if acc.0.is_empty() {
                    acc.1 = c.position;
                }
                acc.0.push(c.character);
                acc
            });
        let comment_regex: &Regex = &regexes.comment;
        let buffer_string: std::borrow::Cow<'_, str> = comment_regex.replace_all(&raw_buffer.0, "");
        let buffer_string: &str = buffer_string.trim();
        let buffer: (&str, Position) = (buffer_string, raw_buffer.1);

        match &buffer {
            _ if buffer.0.starts_with(Self::BLOCK_END_CHAR) => {
                return Self::parse_block_end(&file_info, buffer)
            }
            _ if buffer.0.starts_with(Keyword::OBJECT_STR) => {
                return Self::parse_object(&file_info, buffer, &regexes.valid_character)
            }
            _ if buffer.0.starts_with(Keyword::ANIMATION_STR) => {
                return Self::parse_animation(&file_info, buffer, &regexes.valid_character)
            }
            _ => {}
        }
        let mut words = buffer.0.split(' ');

        let first_word = words.next().ok_or(CompileError::new(
            &file_info,
            buffer.1,
            ErrorType::LineEmpty(buffer.0.to_string()),
        ))?;
        let keyword: &str;
        let numbers: NumberSet;
        if !NumberType::has_prefix(first_word) {
            keyword = first_word;
            numbers = current_numbers
        } else {
            let first_number: Number = first_word
                .parse()
                .map_err(|err| CompileError::new(&file_info, buffer.1, err))?;

            let second_word = words.next().ok_or(CompileError::new(
                &file_info,
                buffer.1,
                ErrorType::LineEmpty(buffer.0.to_string()),
            ))?;

            if !NumberType::has_prefix(second_word) {
                keyword = second_word;
                numbers = match first_number.number_type {
                    NumberType::Delay => {
                        NumberSet::new(first_number.value, current_numbers.duration)
                    }
                    NumberType::Duration => {
                        NumberSet::new(current_numbers.delay, first_number.value)
                    }
                }
            } else {
                let second_number: Number = second_word
                    .parse()
                    .map_err(|err| CompileError::new(&file_info, buffer.1, err))?;
                numbers = NumberSet::new_unordered(first_number, second_number)?;
                keyword = words.next().ok_or(CompileError::new(
                    &file_info,
                    buffer.1,
                    ErrorType::LineEmpty(buffer.0.to_string()),
                ))?;
            }
        }

        dbg!((keyword, numbers));

        todo!()
    }

    fn parse_block_end(file_info: &FileInfo, buffer: (&str, Position)) -> AResult<Statement> {
        assert!(buffer.0.starts_with(Self::BLOCK_END_CHAR));
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

    fn parse_object(
        file_info: &FileInfo,
        buffer: (&str, Position),
        valid_char_regex: &Regex,
    ) -> AResult<Statement> {
        assert!(buffer.0.starts_with(Keyword::OBJECT_STR));
        let (keyword, argument) = buffer.0.split_once(' ').ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::KeywordWithoutArguments(
                Keyword::OBJECT_STR.to_string(),
                buffer.0.to_string(),
            ),
        ))?;
        ensure!(
            !valid_char_regex.is_match(argument),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidCharacters(argument.to_string())
            )
        );

        Ok(Self::ObjectName(argument.to_string()))
    }

    fn parse_animation(
        file_info: &FileInfo,
        buffer: (&str, Position),
        valid_char_regex: &Regex,
    ) -> AResult<Statement> {
        assert!(buffer.0.starts_with(Keyword::ANIMATION_STR));
        let (keyword, argument) = buffer.0.split_once(' ').ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::KeywordWithoutArguments(
                Keyword::ANIMATION_STR.to_string(),
                buffer.0.to_string(),
            ),
        ))?;
        ensure!(keyword == Keyword::END_STR);
        ensure!(
            !valid_char_regex.is_match(argument),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidCharacters(argument.to_string())
            )
        );

        Ok(Self::AnimationName(argument.to_string()))
    }

    fn parse_end(
        file_info: &FileInfo,
        number: Number,
        buffer: (&str, Position),
    ) -> AResult<Statement> {
        assert!(buffer.0.contains(Keyword::END_STR));
        ensure!(
            number.number_type == NumberType::Delay,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectNumberType(number, NumberType::Delay)
            )
        );
        let words: Vec<_> = buffer.0.split(' ').collect();

        ensure!(
            words.len() == 2,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::TooManyWords(buffer.0.to_string(), 2, words.len())
            )
        );
        Ok(Self::End(number.value))
    }
}

enum Keyword {
    Translate,
    Rotate,
    Scale,
    //Spawn,
}
impl Keyword {
    pub const TRANSLATE_STR: &'static str = "translate";
    pub const ROTATE_STR: &'static str = "rotate";
    pub const SCALE_STR: &'static str = "scale";
    pub const OBJECT_STR: &'static str = "object";
    pub const ANIMATION_STR: &'static str = "anim";
    pub const END_STR: &'static str = "end";
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
