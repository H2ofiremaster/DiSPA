use std::{collections::HashMap, str::FromStr};

use crate::{
    errors::{CompileError, CompileErrorType as ErrorType, GenericError},
    objects::{
        Entity, Number, NumberSet, NumberType, Position, Regexes, Rotation, Scale,
        SimpleTransformation, TrackedChar, Translation,
    },
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
        let mut entity_map: HashMap<String, Entity> = HashMap::new();
        let mut statements: Vec<Statement> = Vec::new();

        loop {
            let current_numbers = *block_queue.last().ok_or(GenericError::BlockQueueEmpty)?;
            let next_statement = Statement::parse_from_file(
                &file_info,
                contents,
                current_numbers,
                &regexes,
                &mut entity_map,
            )?;

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
    Keyword(NumberSet, Entity, KeywordStatement),
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
        entities: &mut HashMap<String, Entity>,
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

        let first_word = words.next().ok_or(CompileError::new(
            file_info,
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
                .map_err(|err| CompileError::new(file_info, buffer.1, err))?;

            let second_word = words.next().ok_or(CompileError::new(
                file_info,
                buffer.1,
                ErrorType::LineEmpty(buffer.0.to_string()),
            ))?;

            if !NumberType::has_prefix(second_word) {
                if second_word.contains(Keyword::END_STR) {
                    return Self::parse_end(file_info, first_number, buffer);
                }

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
                    .map_err(|err| CompileError::new(file_info, buffer.1, err))?;

                numbers = NumberSet::new_unordered(first_number, second_number)
                    .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
                keyword = words.next().ok_or(CompileError::new(
                    file_info,
                    buffer.1,
                    ErrorType::LineEmpty(buffer.0.to_string()),
                ))?;
            }
        }

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
                buffer.1,
                ErrorType::IncorrectSeparator(buffer.0.to_string(), ';')
            )
        );
        match keyword
            .parse()
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?
        {
            Keyword::Translate => Self::parse_transformation::<Translation>(
                file_info, buffer, numbers, &arguments, entities,
            ),
            Keyword::Rotate => Self::parse_transformation::<Rotation>(
                file_info, buffer, numbers, &arguments, entities,
            ),
            Keyword::Scale => Self::parse_transformation::<Scale>(
                file_info, buffer, numbers, &arguments, entities,
            ),
            Keyword::Spawn => todo!(),
            Keyword::Item => todo!(),
            Keyword::Block => todo!(),
            Keyword::Text => todo!(),
        }
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

    fn parse_end(file_info: &FileInfo, number: Number, buffer: Buffer) -> AResult<Statement> {
        assert!(buffer.0.contains(Keyword::END_STR));
        ensure!(
            buffer.0.ends_with(';'),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectSeparator(buffer.0.to_string(), ';')
            )
        );
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
        Ok(Self::End(number.value))
    }

    fn parse_transformation<T>(
        file_info: &FileInfo,
        buffer: Buffer,
        numbers: NumberSet,
        arguments: &[&str],
        entities: &mut HashMap<String, Entity>,
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
        let entity_name = arguments[0].to_owned();
        let entity = entities
            .entry(entity_name.clone())
            .or_insert(Entity::new(entity_name));
        let coordinates: (f32, f32, f32) = (
            Self::parse_coordinate(
                file_info,
                buffer.1,
                arguments[1],
                T::get_x(entity.transformation),
            )?,
            Self::parse_coordinate(
                file_info,
                buffer.1,
                arguments[2],
                T::get_y(entity.transformation),
            )?,
            Self::parse_coordinate(
                file_info,
                buffer.1,
                arguments[3],
                T::get_z(entity.transformation),
            )?,
        );
        let t: T = T::from(coordinates);
        entity.transformation = t.transform(entity.transformation);
        Ok(Self::Keyword(numbers, entity.clone(), t.to_statement()))
    }

    fn parse_coordinate(
        info: &FileInfo,
        pos: Position,
        coordinate: &str,
        current: f32,
    ) -> AResult<f32> {
        let coordinate = coordinate.to_string();
        if coordinate.starts_with('~') {
            Ok(coordinate
                .replace("~-", "-0")
                .replace('~', "0")
                .parse::<f32>()
                .map_err(|err| {
                    CompileError::new(
                        info,
                        pos,
                        ErrorType::InvalidCoordinate(coordinate.clone(), err),
                    )
                })?
                + current)
        } else {
            Ok(coordinate.parse::<f32>().map_err(|err| {
                CompileError::new(
                    info,
                    pos,
                    ErrorType::InvalidCoordinate(coordinate.clone(), err),
                )
            })?)
        }
    }

    fn parse_spawn(
        info: &FileInfo,
        buffer: Buffer,
        numbers: NumberSet,
        arguments: &[&str],
    ) -> AResult<Statement> {
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
    //Spawn(Entity, EntityType),
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
        let mut entity_map = HashMap::new();

        let test_1 = Statement::parse_from_file(
            &file_info,
            &mut contents,
            current_numbers,
            &regexes,
            &mut entity_map,
        );
        println!("Test 1: {test_1:?}");
        let test_2 = Statement::parse_from_file(
            &file_info,
            &mut contents,
            current_numbers,
            &regexes,
            &mut entity_map,
        );
        println!("Test 2: {test_2:?}");

        let test_3 = Statement::parse_from_file(
            &file_info,
            &mut contents,
            current_numbers,
            &regexes,
            &mut entity_map,
        );
        println!("Test 3: {test_3:?}");

        let test_4 = Statement::parse_from_file(
            &file_info,
            &mut contents,
            current_numbers,
            &regexes,
            &mut entity_map,
        );
        println!("Test 4: {test_4:?}");

        let test_5 = Statement::parse_from_file(
            &file_info,
            &mut contents,
            current_numbers,
            &regexes,
            &mut entity_map,
        );
        println!("Test 5: {test_5:?}");

        // for c in contents {
        //     println!("{}", c);
        // }
    }
}
