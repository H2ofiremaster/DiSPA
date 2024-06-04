use crate::{
    errors::{CompileError, CompileErrorType as ErrorType},
    objects::{BlockState, Entity, Position, Regexes, Rotation, Scale, TrackedChar, Translation},
};

use anyhow::{ensure, Result as AResult};
use regex::Regex;

macro_rules! arg_count {
    (==$e:expr, $data:expr) => {
        ensure!(
            $data.arguments.len() == $e,
            $data.compile_error(ErrorType::IncorrectArgumentCount(
                $data.buffer.0,
                $e,
                $data.arguments.len()
            ))
        )
    };
    (>=$e:expr, $data:expr) => {
        ensure!(
            $data.arguments.len() >= $e,
            $data.compile_error(ErrorType::IncorrectArgumentCount(
                $data.buffer.0,
                $e,
                $data.arguments.len()
            ))
        )
    };
}

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

#[derive(Debug, Clone, Copy)]
struct StatementData<'a> {
    file_info: &'a FileInfo,
    buffer: Buffer<'a>,
    arguments: &'a [&'a str],
    name_regex: &'a Regex,
}
impl<'a> StatementData<'a> {
    fn compile_error(&self, error_type: ErrorType) -> CompileError {
        CompileError::new(self.file_info, self.buffer.1, error_type)
    }
    fn compile_error_offset(&self, offset: usize, error_type: ErrorType) -> CompileError {
        CompileError::new(self.file_info, self.buffer.1 + offset, error_type)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ObjectName(String, String),
    Wait(u32),
    Translate(Entity, Translation, u32),
    Rotate(Entity, Rotation, u32),
    Scale(Entity, Scale, u32),
    Spawn(Entity, String, Entity),
    Item(Entity, String),
    Block(Entity, BlockState),
    Text(Entity, String),
    Teleport(Entity, f32, f32, f32),
    Raw(String, bool),
    Empty,
}
impl Statement {
    const RAW_COMMAND_PREFIX: char = '/';

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
        let mut chars = buffer.0.chars();
        if Self::is_raw(chars.next()) {
            let delayed: bool = !Self::is_raw(chars.next());
            return Ok(Self::Raw(
                buffer
                    .0
                    .trim_start_matches(Self::RAW_COMMAND_PREFIX)
                    .to_string(),
                delayed,
            ));
        }
        let mut words = buffer.0.split(' ');
        let keyword = words.next().ok_or_else(|| {
            CompileError::new(file_info, buffer.1, ErrorType::LineEmpty(buffer.0))
        })?;
        let arguments: Vec<_> = words.collect();

        let buffer: Buffer = (buffer.0, buffer.1 + keyword.len());

        let data = StatementData {
            file_info,
            buffer,
            arguments: &arguments,
            name_regex: &regexes.name,
        };

        match keyword.try_into().map_err(|err| data.compile_error(err))? {
            Keyword::Object => Self::parse_object(data),
            Keyword::Wait => Self::parse_wait(data),

            Keyword::Translate => Self::parse_translation(data),
            Keyword::Rotate => Self::parse_rotation(data),
            Keyword::Scale => Self::parse_scale(data),

            Keyword::Spawn => Self::parse_spawn(data),
            Keyword::Item => Self::parse_item(data),
            Keyword::Block => Self::parse_block(data),
            Keyword::Text => Self::parse_text(data),
            Keyword::Teleport => Self::parse_teleport(data),
        }
    }

    const fn is_raw(char: Option<char>) -> bool {
        match char {
            Some(c) => c == Self::RAW_COMMAND_PREFIX,
            None => false,
        }
    }

    fn parse_object(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(== 1, data);
        let argument = arguments[0];
        let (object_name, animation_name) = argument
            .split_once(':')
            .ok_or_else(|| data.compile_error(ErrorType::NoAnimationName(argument)))?;
        ensure!(
            name_regex.is_match(object_name),
            data.compile_error(ErrorType::InvalidCharacters(object_name))
        );
        ensure!(
            name_regex.is_match(animation_name),
            data.compile_error(ErrorType::InvalidCharacters(animation_name))
        );

        Ok(Self::ObjectName(
            object_name.to_string(),
            animation_name.to_string(),
        ))
    }

    fn parse_wait(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        arg_count!(== 1, data);
        let wait_duration: u32 = arguments[0]
            .parse()
            .map_err(|err| data.compile_error(ErrorType::InvalidInt(arguments[0], err)))?;
        Ok(Self::Wait(wait_duration))
    }

    fn parse_translation(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(== 5, data);
        let entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;
        let position: Vector = Self::parse_coordinates(arguments[1], arguments[2], arguments[3])
            .map_err(|err| data.compile_error(err))?;
        let duration: u32 = arguments[4]
            .parse()
            .map_err(|err| data.compile_error(ErrorType::InvalidInt(arguments[4], err)))?;
        let translation = Translation::new(position);
        Ok(Self::Translate(entity, translation, duration))
    }

    fn parse_rotation(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(== 4, data);
        let entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;

        let axis: [f32; 3] =
            Self::parse_axis(arguments[1]).map_err(|err| data.compile_error(err))?;
        let angle: f32 = arguments[2]
            .parse()
            .map_err(|err| data.compile_error(ErrorType::InvalidFloat(arguments[2], err)))?;

        let duration: u32 = arguments[3]
            .parse()
            .map_err(|err| data.compile_error(ErrorType::InvalidInt(arguments[3], err)))?;

        let rotation = Rotation::new(axis, angle);
        Ok(Self::Rotate(entity, rotation, duration))
    }

    fn parse_scale(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(== 5, data);

        let entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;

        let position: Vector = Self::parse_coordinates(arguments[1], arguments[2], arguments[3])
            .map_err(|err| data.compile_error(err))?;

        let duration: u32 = arguments[4]
            .parse()
            .map_err(|err| data.compile_error(ErrorType::InvalidInt(arguments[4], err)))?;

        let scale = Scale::new(position);
        Ok(Self::Scale(entity, scale, duration))
    }

    fn parse_coordinates<'a>(x: &'a str, y: &'a str, z: &'a str) -> Result<Vector, ErrorType<'a>> {
        let x = x
            .parse()
            .map_err(|err| ErrorType::InvalidCoordinate(x, err))?;
        let y = y
            .parse()
            .map_err(|err| ErrorType::InvalidCoordinate(y, err))?;
        let z = z
            .parse()
            .map_err(|err| ErrorType::InvalidCoordinate(z, err))?;
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
            return Err(ErrorType::InvalidAxis(axis_string));
        }
        let axes: Vec<f32> = axis_string
            .strip_prefix('[')
            .ok_or(ErrorType::InvalidAxis(axis_string))?
            .strip_suffix(']')
            .ok_or(ErrorType::InvalidAxis(axis_string))?
            .replace(' ', "")
            .split(',')
            .map(|char| {
                char.parse()
                    .map_err(|_| ErrorType::InvalidAxis(axis_string))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if axes.len() != 3 {
            return Err(ErrorType::InvalidAxis(axis_string));
        }
        Ok([axes[0], axes[1], axes[2]])
    }

    fn parse_spawn(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(== 3, data);
        let source_entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;
        let entity_type = arguments[1];
        ensure!(
            Entity::TYPES.contains(&entity_type),
            data.compile_error(ErrorType::InvalidEntityType(entity_type),),
        );
        let new_entity =
            Entity::new(arguments[2], name_regex).map_err(|err| data.compile_error(err))?;
        Ok(Self::Spawn(
            source_entity,
            entity_type.to_string(),
            new_entity,
        ))
    }

    fn parse_item(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(>= 2, data);

        let entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;
        let item = arguments[1..].join(" ");
        Ok(Self::Item(entity, item))
    }

    fn parse_block(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(>= 2, data);

        let entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;
        let block_state = arguments[1..].join(" ");
        let (id, state): (&str, _) = block_state
            .split_once('[')
            .map_or((&block_state, None), |(id, state)| (id, Some(state)));
        let block = match state {
            None => BlockState::new(id.to_string(), Vec::new()),
            Some(state) => {
                let state = state.strip_suffix(']').ok_or_else(|| {
                    data.compile_error_offset(
                        id.len() + state.len(),
                        ErrorType::InvalidState(state),
                    )
                })?;
                let states: Vec<_> = state
                    .split(',')
                    .map(|part| {
                        part.trim()
                            .split_once('=')
                            .map(|(name, value)| (name.to_string(), value.to_string()))
                            .ok_or_else(|| {
                                data.compile_error_offset(id.len(), ErrorType::InvalidState(state))
                            })
                    })
                    .collect();
                let states = crate::collect_errors(states)?;
                BlockState::new(id.to_string(), states)
            }
        };
        Ok(Self::Block(entity, block))
    }

    fn parse_text(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(>= 2, data);

        let entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;
        let text = arguments[1..].join(" ");
        Ok(Self::Text(entity, text))
    }

    fn parse_teleport(data: StatementData) -> AResult<Self> {
        let arguments = data.arguments;
        let name_regex = data.name_regex;
        arg_count!(== 4, data);

        let entity =
            Entity::new(arguments[0], name_regex).map_err(|err| data.compile_error(err))?;
        let coords = arguments[1..=3]
            .iter()
            .map(|coord| {
                coord
                    .parse()
                    .map_err(|err| data.compile_error(ErrorType::InvalidFloat(coord, err)))
            })
            .collect::<Result<Vec<f32>, _>>()?;
        let [x, y, z] = coords.as_slice() else {
            unreachable!()
        };

        Ok(Self::Teleport(entity, *x, *y, *z))
    }
}

#[derive(Debug, Clone, Copy)]
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
    Teleport,
}
impl<'a> TryFrom<&'a str> for Keyword {
    type Error = ErrorType<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let result = match value.to_lowercase().as_str() {
            "object" | "anim" => Self::Object,
            "wait" | "delay" => Self::Wait,
            "translate" | "move" | "m" => Self::Translate,
            "rotate" | "turn" | "r" => Self::Rotate,
            "scale" | "size" | "s" => Self::Scale,
            "spawn" => Self::Spawn,
            "item" => Self::Item,
            "block" => Self::Block,
            "text" => Self::Text,
            "teleport" | "tp" => Self::Teleport,
            _ => return Err(ErrorType::InvalidKeyword(value)),
        };
        Ok(result)
    }
}

fn get_buffer_string(line: &[TrackedChar]) -> (String, Position) {
    let mut quoted: bool = false;
    assert_ne!(line.len(), 0);
    let raw: bool = line[0].character == Statement::RAW_COMMAND_PREFIX;
    let pos: Position = line[0].position;
    let string: String = line
        .iter()
        .map(|line| line.character)
        .take_while(|&char| {
            if char == '"' {
                quoted = !quoted;
            }
            char != '#' || quoted || raw
        })
        .collect();
    (string.trim().to_string(), pos)
}

#[allow(unused_imports, clippy::missing_const_for_fn, clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
}
