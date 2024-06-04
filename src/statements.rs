use std::str::FromStr;

use crate::{
    errors::{CompileError, CompileErrorType as ErrorType},
    objects::{Entity, Position, Regexes, Rotation, Scale, TrackedChar, Translation},
};

use anyhow::{ensure, Result as AResult};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub eof: TrackedChar,
}
impl FileInfo {
    pub const fn new(path: String, eof: TrackedChar) -> Self {
        Self { path, eof }
    }
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
impl Program {
    pub fn parse_from_file(file_info: &FileInfo, contents: &[TrackedChar]) -> AResult<Self> {
        let regexes = Regexes::new()?;
        let statements: Vec<AResult<Statement>> = contents
            .split(|char| char.character == '\n')
            .filter(|line| !line.is_empty())
            .map(|line| Statement::parse_from_file(file_info, line, &regexes))
            .collect();

        Ok(Self {
            statements: crate::collect_errors(statements)?,
        })
    }
}

pub type Vector = (f32, f32, f32);
type Buffer<'a> = (&'a str, Position);

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ObjectName(String, String),
    Wait(u32),
    Translate(Entity, Translation, u32),
    Rotate(Entity, Rotation, u32),
    Scale(Entity, Scale, u32),
    Spawn {
        source: Entity,
        entity_type: String,
        new: Entity,
    },
    Empty,
}
impl Statement {
    fn parse_from_file(
        file_info: &FileInfo,
        line: &[TrackedChar],
        regexes: &Regexes,
    ) -> AResult<Self> {
        let (buffer_string, buffer_pos) = get_buffer_string(line);
        let buffer: Buffer = (buffer_string.trim(), buffer_pos);
        if buffer.0.is_empty() {
            return Ok(Self::Empty);
        }
        let mut words = buffer.0.split(' ');
        let keyword = words.next().ok_or_else(|| {
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::LineEmpty(buffer.0.to_string()),
            )
        })?;
        let arguments: Vec<_> = words.collect();

        let buffer: Buffer = (buffer.0, buffer.1 + keyword.len());

        match keyword
            .parse()
            .map_err(|err| CompileError::new(file_info, buffer.1 - keyword.len(), err))?
        {
            Keyword::Object => Self::parse_object(file_info, buffer, &arguments, &regexes.name),
            Keyword::Wait => Self::parse_wait(file_info, buffer, &arguments),

            Keyword::Translate => {
                Self::parse_translation(file_info, buffer, &arguments, &regexes.name)
            }
            Keyword::Rotate => Self::parse_rotation(file_info, buffer, &arguments, &regexes.name),
            Keyword::Scale => Self::parse_scale(file_info, buffer, &arguments, &regexes.name),

            Keyword::Spawn => Self::parse_spawn(file_info, buffer, &arguments, &regexes.name),
            Keyword::Item => Self::parse_item(file_info, buffer, &arguments),
            Keyword::Block => Self::parse_block(file_info, buffer, &arguments),
            Keyword::Text => Self::parse_text(file_info, buffer, &arguments),
        }
    }

    fn parse_object(
        file_info: &FileInfo,
        buffer: Buffer,
        arguments: &[&str],
        name_regex: &Regex,
    ) -> AResult<Self> {
        ensure!(
            arguments.len() == 1,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 1, arguments.len()),
            )
        );
        let argument = arguments[0];
        let (object_name, animation_name) = argument.split_once(':').ok_or_else(|| {
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::NoAnimationName(argument.to_string()),
            )
        })?;
        ensure!(
            name_regex.is_match(argument),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidCharacters(argument.to_string())
            )
        );

        Ok(Self::ObjectName(
            object_name.to_string(),
            animation_name.to_string(),
        ))
    }

    fn parse_wait(file_info: &FileInfo, buffer: Buffer, arguments: &[&str]) -> AResult<Self> {
        ensure!(
            arguments.len() == 1,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 1, arguments.len())
            )
        );
        let wait_duration: u32 = arguments[0].parse().map_err(|err| {
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidInt(buffer.0.to_string(), err),
            )
        })?;
        Ok(Self::Wait(wait_duration))
    }

    fn parse_translation(
        file_info: &FileInfo,
        buffer: Buffer,
        arguments: &[&str],
        name_regex: &Regex,
    ) -> AResult<Self> {
        ensure!(
            arguments.len() == 5,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 5, arguments.len())
            )
        );
        let entity = Entity::new(arguments[0].to_string(), name_regex)
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let duration: u32 = arguments[1].parse().map_err(|err| {
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidInt(buffer.0.to_string(), err),
            )
        })?;
        let position: Vector = Self::parse_coordinates(arguments[2], arguments[3], arguments[4])
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let translation = Translation::new(position);
        Ok(Self::Translate(entity, translation, duration))
    }

    fn parse_rotation(
        file_info: &FileInfo,
        buffer: Buffer,
        arguments: &[&str],
        name_regex: &Regex,
    ) -> AResult<Self> {
        ensure!(
            arguments.len() == 4,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 5, arguments.len())
            )
        );
        let entity = Entity::new(arguments[0].to_string(), name_regex)
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let duration: u32 = arguments[1].parse().map_err(|err| {
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidInt(arguments[1].to_string(), err),
            )
        })?;
        let axis = Self::parse_axis(arguments[2])
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let angle: f32 = arguments[3].parse().map_err(|err| {
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidFloat(arguments[3].to_string(), err),
            )
        })?;
        let rotation = Rotation::new(axis, angle);
        Ok(Self::Rotate(entity, rotation, duration))
    }

    fn parse_scale(
        file_info: &FileInfo,
        buffer: Buffer,
        arguments: &[&str],
        name_regex: &Regex,
    ) -> AResult<Self> {
        ensure!(
            arguments.len() == 5,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 5, arguments.len())
            )
        );
        let entity = Entity::new(arguments[0].to_string(), name_regex)
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let duration: u32 = arguments[1].parse().map_err(|err| {
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidInt(buffer.0.to_string(), err),
            )
        })?;
        let position: Vector = Self::parse_coordinates(arguments[2], arguments[3], arguments[4])
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let scale = Scale::new(position);
        Ok(Self::Scale(entity, scale, duration))
    }

    fn parse_coordinates(x: &str, y: &str, z: &str) -> Result<Vector, ErrorType> {
        let x = x
            .parse()
            .map_err(|err| ErrorType::InvalidCoordinate(x.to_string(), err))?;
        let y = y
            .parse()
            .map_err(|err| ErrorType::InvalidCoordinate(y.to_string(), err))?;
        let z = z
            .parse()
            .map_err(|err| ErrorType::InvalidCoordinate(z.to_string(), err))?;
        Ok((x, y, z))
    }

    fn parse_axis(axis_string: &str) -> Result<[f32; 3], ErrorType> {
        match axis_string {
            "x" => return Ok([1.0, 0.0, 0.0]),
            "y" => return Ok([0.0, 1.0, 0.0]),
            "z" => return Ok([0.0, 0.0, 1.0]),
            _ => {}
        }
        if !(axis_string.starts_with('[') && axis_string.ends_with(']')) {
            return Err(ErrorType::InvalidAxis(axis_string.to_string()));
        }
        let axes: Vec<f32> = axis_string
            .strip_prefix('[')
            .ok_or_else(|| ErrorType::InvalidAxis(axis_string.to_string()))?
            .strip_suffix(']')
            .ok_or_else(|| ErrorType::InvalidAxis(axis_string.to_string()))?
            .replace(' ', "")
            .split(',')
            .map(|char| {
                char.parse()
                    .map_err(|_| ErrorType::InvalidAxis(axis_string.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if axes.len() != 3 {
            return Err(ErrorType::InvalidAxis(axis_string.to_string()));
        }
        Ok([axes[0], axes[1], axes[2]])
    }
    fn parse_spawn(
        file_info: &FileInfo,
        buffer: Buffer,
        arguments: &[&str],
        name_regex: &Regex,
    ) -> AResult<Self> {
        ensure!(
            arguments.len() == 3,
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::IncorrectArgumentCount(buffer.0.to_string(), 3, arguments.len())
            )
        );
        let source_entity = Entity::new(arguments[0].to_string(), name_regex)
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        let entity_type = arguments[1];
        ensure!(
            Entity::TYPES.contains(&entity_type),
            CompileError::new(
                file_info,
                buffer.1,
                ErrorType::InvalidEntityType(entity_type.to_string()),
            ),
        );
        let new_entity = Entity::new(arguments[2].to_string(), name_regex)
            .map_err(|err| CompileError::new(file_info, buffer.1, err))?;
        Ok(Self::Spawn {
            source: source_entity,
            entity_type: entity_type.to_string(),
            new: new_entity,
        })
    }

    fn parse_item(info: &FileInfo, buffer: Buffer, arguments: &[&str]) -> AResult<Self> {
        todo!()
    }

    fn parse_block(info: &FileInfo, buffer: Buffer, arguments: &[&str]) -> AResult<Self> {
        todo!()
    }

    fn parse_text(info: &FileInfo, buffer: Buffer, arguments: &[&str]) -> AResult<Self> {
        todo!()
    }
}

enum Keyword {
    Object,
    Wait,
    Translate,
    Rotate,
    Scale,
    Spawn,
    Item,
    Block,
    Text,
}
impl FromStr for Keyword {
    type Err = ErrorType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s.to_lowercase().as_str() {
            "object" | "anim" => Self::Object,
            "wait" | "delay" => Self::Wait,
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

fn get_buffer_string(line: &[TrackedChar]) -> (String, Position) {
    let mut quoted = false;
    assert_ne!(line.len(), 0);
    let pos: Position = line[0].position;
    let string: String = line
        .iter()
        .map(|line| line.character)
        .take_while(|&char| {
            if char == '"' {
                quoted = !quoted;
            }
            char != '#' || quoted
        })
        .collect();
    (string.trim().to_string(), pos)
}

#[allow(unused_imports, clippy::missing_const_for_fn, clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_test() {
        let test_str = "\
        abcde#abcde\n\
        abcde\"abc_abc\"abcde\n\
        abcde\"abc#abc\"abcde\n\
        abcde\"abc#abc\"abc#de\n\
        ";
        let tracked = crate::file_reader::to_tracked(test_str)
            .split(|char| char.character == '\n')
            .map(<[TrackedChar]>::to_vec)
            .collect::<Vec<_>>();

        assert_eq!(get_buffer_string(&tracked[0]).0, "abcde".to_string());
        assert_eq!(
            get_buffer_string(&tracked[1]).0,
            "abcde\"abc_abc\"abcde".to_string()
        );
        assert_eq!(
            get_buffer_string(&tracked[2]).0,
            "abcde\"abc#abc\"abcde".to_string()
        );
        assert_eq!(
            get_buffer_string(&tracked[3]).0,
            "abcde\"abc#abc\"abc".to_string()
        );
    }

    #[test]
    fn statement_test() {
        let test_program = "\
        move test 20 0 1 0\n\
        turn test 20 y 90\n\
        size test 20 2 2 2\n\
        wait 40\n\
        spawn test block_display test2\n\
        ";
        let tracked = crate::file_reader::to_tracked(test_program);
        let full_program = Program::parse_from_file(
            &FileInfo {
                path: String::new(),
                eof: TrackedChar::new(6, test_program.len(), '\n'),
            },
            &tracked,
        )
        .unwrap();
        let regexes = Regexes::new().unwrap();

        // dbg!(&full_program);

        assert_eq!(full_program.statements.len(), 5);
        assert_eq!(
            full_program.statements[0],
            Statement::Translate(
                Entity::new("test".to_string(), &regexes.name).unwrap(),
                Translation::new((0.0, 1.0, 0.0)),
                20
            )
        );
        assert_eq!(
            full_program.statements[1],
            Statement::Rotate(
                Entity::new("test".to_string(), &regexes.name).unwrap(),
                Rotation::new([0.0, 1.0, 0.0], 90.0),
                20
            )
        );
        assert_eq!(
            full_program.statements[2],
            Statement::Scale(
                Entity::new("test".to_string(), &regexes.name).unwrap(),
                Scale::new((2.0, 2.0, 2.0)),
                20
            )
        );
        assert_eq!(full_program.statements[3], Statement::Wait(40));
        assert_eq!(
            full_program.statements[4],
            Statement::Spawn {
                source: Entity::new("test".to_string(), &regexes.name).unwrap(),
                entity_type: "block_display".to_string(),
                new: Entity::new("test2".to_string(), &regexes.name).unwrap(),
            }
        );
    }
}
