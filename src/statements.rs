use std::str::{FromStr, Split};

use crate::{
    errors::{CompileError, CompileErrorType as ErrorType, GenericError},
    objects::*,
};

use anyhow::{ensure, Result as AResult};
use itertools::Itertools;
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

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
impl Program {
    pub fn parse_from_file<I: Iterator<Item = TrackedChar>>(
        file_info: FileInfo,
        contents: &mut I,
    ) -> AResult<Self> {
        Ok(Program {
            statements: Self::parse_statements(file_info, contents, Regexes::new()?)?,
        })
    }
    fn parse_statements<I: Iterator<Item = TrackedChar>>(
        file_info: FileInfo,
        contents: &mut I,
        regexes: Regexes,
    ) -> AResult<Vec<Statement>> {
        let mut block_queue = vec![NumberSet::default()];
        let mut statements: Vec<Statement> = Vec::new();

        loop {
            let current_numbers = *block_queue.last().ok_or(GenericError::BlockQueueEmpty)?;
            let next_statement =
                Statement::parse_from_file(&file_info, contents, current_numbers, &regexes)?;

            match next_statement {
                Statement::Block(number_set) => {
                    block_queue.push(number_set);
                }
                Statement::BlockEnd => {
                    block_queue.pop();
                }
                Statement::EndOfFile => {
                    break;
                }
                _ => {}
            };
            statements.push(next_statement);
        }
        Ok(statements)
    }
}

type Buffer<'a> = (&'a str, Position);

#[derive(Debug, Clone)]
pub enum Statement {
    ObjectName(String),
    AnimationName(String),
    Block(NumberSet),
    BlockEnd,
    Keyword(NumberSet, String, KeywordStatement),
    End(u32),
    EndOfFile,
}
impl Statement {
    const STATEMENT_END_CHAR: char = ';';
    const BLOCK_START_CHAR: char = '{';
    const BLOCK_END_CHAR: char = '}';

    fn parse_from_file<I: Iterator<Item = TrackedChar>>(
        file_info: &FileInfo,
        contents: &mut I,
        current_numbers: NumberSet,
        regexes: &Regexes,
    ) -> AResult<Self> {
        let (buffer_string, buffer_pos) = get_buffer_string(
            contents,
            &[
                Self::STATEMENT_END_CHAR,
                Self::BLOCK_START_CHAR,
                Self::BLOCK_END_CHAR,
            ],
        );
        let buffer: Buffer = (buffer_string.trim(), buffer_pos);

        match &buffer {
            _ if buffer.0.is_empty() => return Ok(Self::EndOfFile),
            _ if buffer.0.starts_with(Self::BLOCK_END_CHAR) => {
                return Self::parse_block_end(file_info, buffer);
            }
            _ if buffer.0.starts_with(Keyword::OBJECT_STR) => {
                return Self::parse_name_declaration(
                    file_info,
                    buffer,
                    &regexes.valid_character,
                    Keyword::OBJECT_STR,
                    Self::ObjectName,
                );
            }
            _ if buffer.0.starts_with(Keyword::ANIMATION_STR) => {
                return Self::parse_name_declaration(
                    file_info,
                    buffer,
                    &regexes.valid_character,
                    Keyword::ANIMATION_STR,
                    Self::AnimationName,
                );
            }
            _ => {}
        }
        let mut words = buffer.0.split(' ');

        let (keyword, numbers) =
            Self::parse_numbers(&mut words, file_info, &buffer, current_numbers)?;

        let arguments: Vec<_> = words
            .map(|w| {
                if w.ends_with(';') {
                    return remove_last_char(w);
                }
                w
            })
            .collect();

        if keyword == Self::BLOCK_START_CHAR.to_string() {
            return Self::parse_block_start(file_info, buffer, numbers);
        }
        ensure!(
            buffer.0.ends_with(';'),
            CompileError::new(
                file_info,
                buffer.1 + buffer.0.len(),
                ErrorType::IncorrectSeparator(buffer.0.to_string(), ';')
            )
        );
        let buffer: Buffer = (buffer.0, buffer.1 + keyword.len());

        if keyword.contains(Keyword::END_STR) {
            return Self::parse_end(file_info, numbers.delay, buffer);
        }

        match keyword
            .parse()
            .map_err(|err| CompileError::new(file_info, buffer.1 - keyword.len(), err))?
        {
            Keyword::Translate => {
                Self::parse_transformation::<Translation>(file_info, buffer, numbers, &arguments)
            }
            Keyword::Rotate => {
                Self::parse_transformation::<Rotation>(file_info, buffer, numbers, &arguments)
            }
            Keyword::Scale => {
                Self::parse_transformation::<Scale>(file_info, buffer, numbers, &arguments)
            }
            Keyword::Spawn => Self::parse_spawn(file_info, buffer, numbers.delay, &arguments),
            Keyword::Item => Self::parse_item(file_info, buffer, numbers, &arguments),
            Keyword::Block => Self::parse_block(file_info, buffer, numbers, &arguments),
            Keyword::Text => Self::parse_text(file_info, buffer, numbers, &arguments),
        }
    }

    fn parse_numbers(
        words: &mut Split<'_, char>,
        file_info: &FileInfo,
        buffer: &Buffer,
        current_numbers: NumberSet,
    ) -> AResult<(String, NumberSet)> {
        let first_word = words.next().ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::LineEmpty(buffer.0.to_string()),
        ))?;
        if !NumberType::has_prefix(first_word) {
            return Ok((first_word.to_string(), current_numbers));
        }

        let first_number: Number = first_word
            .parse()
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;

        let second_word = words.next().ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::LineEmpty(buffer.0.to_string()),
        ))?;
        if !NumberType::has_prefix(second_word) {
            let numbers = NumberSet::new_with_default(first_number, current_numbers);
            return Ok((second_word.to_string(), numbers));
        }
        let second_number: Number = second_word
            .parse()
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;

        let numbers = NumberSet::new_unordered(first_number, second_number)
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let keyword = words.next().ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::LineEmpty(buffer.0.to_string()),
        ))?;
        Ok((keyword.to_string(), numbers))
    }

    fn parse_block_start(
        file_info: &FileInfo,
        buffer: Buffer,
        numbers: NumberSet,
    ) -> AResult<Statement> {
        assert!(buffer.0.contains(Self::BLOCK_START_CHAR));
        let words: Vec<_> = buffer.0.split(' ').collect();

        ensure!(
            words.len() > 1,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 2, words.len())
            )
        );
        ensure!(
            words.len() < 4,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 3, words.len())
            )
        );
        Ok(Self::Block(numbers))
    }

    fn parse_block_end(file_info: &FileInfo, buffer: Buffer) -> AResult<Statement> {
        assert!(buffer.0.starts_with(Self::BLOCK_END_CHAR));
        ensure!(
            buffer.0.len() <= 1,
            CompileError::new(
                file_info,
                buffer.1 + 1,
                ErrorType::TooManyCharacters(buffer.0.to_owned(), 1, buffer.0.len())
            )
        );

        Ok(Self::BlockEnd)
    }

    fn parse_name_declaration<F: FnOnce(String) -> Statement>(
        file_info: &FileInfo,
        buffer: Buffer,
        valid_char_regex: &Regex,
        keyword: &str,
        return_value: F,
    ) -> AResult<Statement> {
        assert!(buffer.0.starts_with(keyword));
        ensure!(
            buffer.0.ends_with(';'),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectSeparator(buffer.0.to_string(), ';')
            )
        );
        let (keyword, argument) = buffer.0.split_once(' ').ok_or(CompileError::new(
            file_info,
            buffer.1,
            ErrorType::KeywordWithoutArguments(keyword.to_string(), buffer.0.to_string()),
        ))?;
        let argument = remove_last_char(argument);

        ensure!(keyword == keyword);
        ensure!(
            valid_char_regex.is_match(argument),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidCharacters(argument.to_string())
            )
        );

        Ok(return_value(argument.to_string()))
    }

    fn parse_end(file_info: &FileInfo, number: u32, buffer: Buffer) -> AResult<Statement> {
        assert!(buffer.0.contains(Keyword::END_STR));
        ensure!(
            buffer.0.ends_with(';'),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectSeparator(buffer.0.to_string(), ';')
            )
        );
        let words: Vec<_> = buffer.0.split(' ').collect();

        ensure!(
            words.len() == 2,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 0, words.len())
            )
        );
        ensure!(
            words
                .last()
                .is_some_and(|s| remove_last_char(s) == Keyword::END_STR),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidKeyword(
                    words
                        .last()
                        .expect("'words' should not be empty due to 'words.len() == 2' passing")
                        .to_string()
                )
            )
        );
        Ok(Self::End(number))
    }

    fn parse_transformation<T>(
        file_info: &FileInfo,
        buffer: Buffer,
        numbers: NumberSet,
        arguments: &[&str],
    ) -> AResult<Statement>
    where
        T: SimpleTransformation + From<(f32, f32, f32)>,
    {
        ensure!(
            arguments.len() == 4,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 4, arguments.len())
            )
        );

        let coordinates: (f32, f32, f32) = (
            Self::parse_coordinate(file_info, buffer.1, arguments[1])?,
            Self::parse_coordinate(file_info, buffer.1, arguments[2])?,
            Self::parse_coordinate(file_info, buffer.1, arguments[3])?,
        );
        let t: T = T::from(coordinates);
        Ok(Self::Keyword(
            numbers,
            arguments[0].to_string(),
            t.to_statement(),
        ))
    }

    fn parse_coordinate(
        file_info: &FileInfo,
        position: Position,
        coordinate: &str,
    ) -> AResult<f32> {
        let coordinate = coordinate.to_string();
        Ok(coordinate.parse::<f32>().map_err(|err| {
            CompileError::new(
                file_info,
                position,
                ErrorType::InvalidCoordinate(coordinate.clone(), err),
            )
        })?)
    }

    fn parse_spawn(
        info: &FileInfo,
        buffer: Buffer,
        delay: u32,
        arguments: &[&str],
    ) -> AResult<Statement> {
        ensure!(
            arguments.len() >= 3,
            CompileError::new(
                info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.1.to_string(), 4, arguments.len())
            )
        );
        ensure!(
            EntityType::TYPES.contains(&arguments[0]),
            CompileError::new(
                info,
                buffer.1,
                ErrorType::InvalidEntityType(arguments[0].to_string())
            )
        );
        if arguments.len() == 3 {}
        todo!()
    }

    fn parse_item(
        info: &FileInfo,
        buffer: Buffer,
        numbers: NumberSet,
        arguments: &[&str],
    ) -> AResult<Statement> {
        todo!()
    }

    fn parse_block(
        info: &FileInfo,
        buffer: Buffer,
        numbers: NumberSet,
        arguments: &[&str],
    ) -> AResult<Statement> {
        todo!()
    }

    fn parse_text(
        info: &FileInfo,
        buffer: Buffer,
        numbers: NumberSet,
        arguments: &[&str],
    ) -> AResult<Statement> {
        todo!()
    }
}

enum Keyword {
    Translate,
    Rotate,
    Scale,
    Spawn,
    Item,
    Block,
    Text,
}
impl Keyword {
    pub const OBJECT_STR: &'static str = "object";
    pub const ANIMATION_STR: &'static str = "anim";
    pub const END_STR: &'static str = "end";
}
impl FromStr for Keyword {
    type Err = ErrorType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s.to_lowercase().as_str() {
            "translate" | "move" | "m" => Self::Translate,
            "rotate" | "turn" | "r" => Self::Rotate,
            "scale" | "size" | "s" => Self::Scale,
            "spawn" => Self::Spawn,
            "item" => Self::Item,
            "block" => Self::Block,
            "text" => Self::Text,
            _ => return Err(ErrorType::InvalidKeyword(s.to_string())),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub enum KeywordStatement {
    Translate(Translation),
    Rotate(Rotation),
    Scale(Scale),
    Spawn(String, (u32, u32, u32)),
}

fn remove_last_char(s: &str) -> &str {
    &s[..s.len() - 1]
}

fn get_buffer_string<I: Iterator<Item = TrackedChar>>(
    iter: &mut I,
    seperators: &[char],
) -> (String, Position) {
    let mut commented: bool = false;
    let mut escaped: bool = false;
    let mut quoted: bool = false;
    iter.filter(|c| {
        commented = match c.character {
            '#' => true,
            '\n' => false,
            _ => commented,
        };
        !commented
    })
    .take_while_inclusive(|c| {
        if is_quoted(c, &mut escaped, &mut quoted) {
            return true;
        }
        !seperators.contains(&c.character)
    })
    .fold((String::new(), Position::default()), |mut acc, c| {
        if acc.0.is_empty() {
            acc.1 = c.position;
        }
        acc.0.push(c.character);
        acc
    })
}

fn is_quoted(c: &TrackedChar, escaped: &mut bool, quoted: &mut bool) -> bool {
    match quoted {
        true => {
            if c.character == '\'' {
                *quoted = false;
            }
        }
        false => {
            if *escaped {
                *escaped = false;
            } else if c.character == '\'' {
                *quoted = true;
            }
        }
    }
    *quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_test() {
        let test_str = "object test;
            # seperate comment
            anim invalid;
            @10 %20 move test 1 0 0; # inline comment
            @10 turn test2 90 0 0;
            @20 %20 move test ~2 ~ ~;";
        let mut contents = crate::file_reader::to_tracked_iter(test_str);
        //println!("{}", contents.map(|c| c.character).collect::<String>());
        //panic!();

        let file_info = FileInfo::new(
            "test/path.dspa".to_string(),
            TrackedChar::new(
                test_str.chars().filter(|&c| c == '\n').count(),
                test_str.lines().last().map(|c| c.len()).unwrap_or(0),
                test_str.chars().last().unwrap_or('\n'),
            ),
        );

        let current_numbers = NumberSet::new(0, 0);
        let regexes = Regexes::new().unwrap();

        let test_1 =
            Statement::parse_from_file(&file_info, &mut contents, current_numbers, &regexes);
    }
}
