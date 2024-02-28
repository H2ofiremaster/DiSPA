use std::fs;

use crate::{
    errors::GenericError,
    statements::{Program, TrackedChar},
};

pub fn parse_file(file_path: &str) -> anyhow::Result<CompiledFile> {
    let contents =
        fs::read_to_string(file_path).map_err(|err| GenericError::InvalidPath(err.to_string()))?;
    let mut iterator = contents
        .lines()
        .enumerate()
        .flat_map(|(line_number, line)| {
            line.chars()
                .enumerate()
                .map(move |(column_number, character)| {
                    TrackedChar::new(line_number, column_number, character)
                })
        });
    Program::parse(file_path, &mut iterator);
    todo!()
}
pub struct CompiledFile {
    pub path: String,
    pub object_name: String,
    pub animation_name: String,
    pub contents: String,
}

/*
use anyhow::{bail, Context};
use anyhow::{ensure, Result as AResult};
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

use crate::errors::{CompileError, NumberSetError};
use crate::objects::Transformation;
use crate::{collect_errors, compiled};


macro_rules! next_element {
    ($elements:expr, $statement:expr, $current:expr) => {
        $elements.next().context(CompileError::TooFewElements(
            $statement.to_string(),
            $current,
            7,
        ))?
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NumberType {
    Delay,
    Duration,
}
impl NumberType {
    const DELAY_PREFIX: char = '@';
    const DURATION_PREFIX: char = '%';
}
impl TryFrom<char> for NumberType {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            Self::DELAY_PREFIX => Ok(Self::Delay),
            Self::DURATION_PREFIX => Ok(Self::Duration),
            _ => bail!("Char '{value}' does not correspond to a NumberType."),
        }
    }
}
impl Display for NumberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberType::Delay => write!(f, "Delay"),
            NumberType::Duration => write!(f, "Duration"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Number {
    number_type: NumberType,
    value: u32,
}
impl FromStr for Number {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut value = value.to_string();

        let number_type = value
            .remove(0)
            .try_into()
            .context(CompileError::InvalidDiscriminator(value.clone()))?;
        let value: u32 = value
            .parse()
            .map_err(|err| CompileError::InvalidNumber(value.clone(), err))?;
        Ok(Number { number_type, value })
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct NumberSet {
    delay: u32,
    duration: u32,
}
impl NumberSet {
    fn new(delay: u32, duration: u32) -> Self {
        Self { delay, duration }
    }

    fn from_vec_with_fallback(
        vec: Vec<Number>,
        fallback: NumberSet,
    ) -> Result<Self, NumberSetError> {
        match vec.len() {
            1 => {
                let number = vec[0];
                let set = match number.number_type {
                    NumberType::Delay => NumberSet::new(number.value, fallback.duration),
                    NumberType::Duration => NumberSet::new(fallback.delay, number.value),
                };
                Ok(set)
            }
            2 => Self::try_from((vec[0], vec[1])),
            3.. => Err(NumberSetError::TooMany(vec.len() as u32)),
            _ => Err(NumberSetError::TooFew(0)),
        }
    }
}
impl TryFrom<(Number, Number)> for NumberSet {
    type Error = NumberSetError;

    fn try_from(value: (Number, Number)) -> Result<Self, Self::Error> {
        let number_set = match (value.0.number_type, value.1.number_type) {
            (NumberType::Delay, NumberType::Duration) => NumberSet {
                delay: value.0.value,
                duration: value.1.value,
            },
            (NumberType::Duration, NumberType::Delay) => NumberSet {
                duration: value.0.value,
                delay: value.1.value,
            },
            _ => return Err(NumberSetError::Duplicate(value.0.number_type)),
        };
        Ok(number_set)
    }
}
impl From<(u32, u32)> for NumberSet {
    fn from(value: (u32, u32)) -> Self {
        NumberSet::new(value.0, value.1)
    }
}

pub struct CompiledFile {
    pub path: String,
    pub object_name: String,
    pub animation_name: String,
    pub contents: String,
}

const COMMENT_PATTERN: &str = r"#.*?\n";
pub fn parse_file(path: &str) -> AResult<CompiledFile> {
    let comment_regex =
        Regex::new(COMMENT_PATTERN).context(CompileError::InvalidRegex(COMMENT_PATTERN))?;

    let file_name = extract_file_name(path)?;

    let raw_code = read_to_string(path).context(CompileError::InvalidPath(path.to_string()))?;
    let commentless_code = comment_regex.replace_all(&raw_code, "");

    let statements: Vec<_> = commentless_code
        .split(';')
        .flat_map(|s| s.split_inclusive(&['{', '}']))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    ensure!(!statements.is_empty(), CompileError::FileEmpty);
    let (object_name, animation_name, statements) = parse_names(statements, &file_name)?;
    let flattened = collapse_blocks(statements)?;
    let mut compiled = compile_statements(flattened, &object_name, &animation_name)?;
    compiled.push(compiled::increment(&object_name, &animation_name));

    Ok(CompiledFile {
        path: path.to_string(),
        object_name,
        animation_name,
        contents: compiled::disclaimer() + &compiled.join("\n"),
    })
}

const NAME_PATTERN: &str = r"^[A-Za-z0-9_\-]*$";
fn parse_names(statements: Vec<String>, file_name: &str) -> AResult<(String, String, Vec<String>)> {
    let name_regex = Regex::new(NAME_PATTERN).context(CompileError::InvalidRegex(NAME_PATTERN))?;

    let object_name = find_statement(&statements, "object", file_name)?;
    ensure!(
        name_regex.is_match(&object_name),
        CompileError::InvalidCharacters("Object name", object_name.clone()),
    );

    let animation_name = find_statement(&statements, "anim", &object_name)?;
    ensure!(
        name_regex.is_match(&animation_name),
        CompileError::InvalidCharacters("Animation name", animation_name.clone())
    );

    Ok((
        object_name,
        animation_name,
        statements
            .into_iter()
            .filter(|s| !s.starts_with("object") && !s.starts_with("anim"))
            .collect(),
    ))
}

fn find_statement(statements: &[String], prefix: &str, default: &str) -> AResult<String> {
    let statement = statements
        .iter()
        .find(|&statement| statement.starts_with(prefix))
        .map(|s| {
            s.split_once(' ')
                .context(CompileError::NotNamed(prefix.to_string()))
                .map(|r| r.1.to_string())
        })
        .transpose()?
        .unwrap_or(default.to_string());
    Ok(statement)
}

fn extract_file_name(path: &str) -> AResult<String> {
    let file_name = Path::new(path)
        .file_stem()
        .context(CompileError::InvalidPath(path.to_string()))?;
    Ok(file_name.to_string_lossy().to_string())
}

fn collapse_blocks(statements: impl IntoIterator<Item = String>) -> AResult<Vec<String>> {
    let mut blocks: Vec<NumberSet> = vec![NumberSet::default()];
    let flattened: Vec<anyhow::Result<String>> = statements
        .into_iter()
        .map(|statement| {
            if statement.contains('{') {
                let previous_block = blocks.last().context(CompileError::BlockQueueEmpty)?;
                blocks.push(parse_block_definition(&statement, *previous_block)?);
                return Ok(String::new());
            }
            if statement == "}" {
                ensure!(blocks.len() > 1, CompileError::UnbalancedBrackets);
                blocks.pop();
                return Ok(String::new());
            }

            let current_block = blocks.last().context(CompileError::BlockQueueEmpty)?;
            let keyword =
                get_keyword(&statement).context(CompileError::NoKeyword(statement.clone()))?;
            let statement_start = statement
                .split_inclusive(keyword)
                .next()
                .context(CompileError::NoKeyword(statement.clone()))?;
            let mut statement = statement.clone();
            if !statement_start.contains(NumberType::DELAY_PREFIX) {
                statement = format!(
                    "{}{} {statement}",
                    NumberType::DELAY_PREFIX,
                    current_block.delay,
                );
            }
            if !statement_start.contains(NumberType::DURATION_PREFIX) {
                statement = format!(
                    "{}{} {statement}",
                    NumberType::DURATION_PREFIX,
                    current_block.duration,
                );
            }
            Ok(statement)
        })
        .filter(|statement| !matches!(statement, Ok(statement) if statement.is_empty()))
        .collect();
    collect_errors(flattened)
}

fn parse_block_definition(definition: &str, previous_block: NumberSet) -> AResult<NumberSet> {
    let numbers: Vec<AResult<Number>> = definition
        .split(' ')
        .filter(|&s| s != "{")
        .map(str::parse)
        .collect();
    let numbers = collect_errors(numbers)?;

    let number_set = NumberSet::from_vec_with_fallback(numbers, previous_block)
        .map_err(|err| CompileError::InvalidBlockDefinition(definition.to_string(), err))?;
    Ok(number_set)
}

fn compile_statements(
    statements: Vec<String>,
    object_name: &str,
    animation_name: &str,
) -> AResult<Vec<String>> {
    let mut entities: HashMap<String, Transformation> = HashMap::new();
    let compiled: Vec<AResult<String>> = statements
        .into_iter()
        .map(|statement| {
            let mut elements = statement.split(' ');

            let NumberSet { delay, duration } = (
                next_element!(elements, statement, 0).parse::<Number>()?,
                next_element!(elements, statement, 1).parse::<Number>()?,
            )
                .try_into()?;
            let keyword = next_element!(elements, statement, 2);
            if keyword == "end" {
                return Ok(compiled::reset(object_name, animation_name, delay));
            }
            let entity_name = next_element!(elements, statement, 3).to_string();
            let entity = entities.entry(entity_name.clone()).or_default();

            let transformation = match keyword {
                "move" | "translate" | "m" => parse_transformation(
                    elements,
                    &statement,
                    entity,
                    Transformation::with_translation,
                )?,
                "turn" | "rotate" | "r" => parse_transformation(
                    elements,
                    &statement,
                    entity,
                    Transformation::with_rotation,
                )?,
                "size" | "scale" | "s" => parse_transformation(
                    elements,
                    &statement,
                    entity,
                    Transformation::with_scale,
                    /* (padding so rustfmt will format this like the others) */
                )?,
                _ => bail!(CompileError::InvalidKeyword(statement.clone())),
            };
            Ok(compiled::transformation(
                object_name,
                animation_name,
                &entity_name,
                delay,
                duration,
                &transformation,
            ))
        })
        .collect();
    collect_errors(compiled)
}

fn parse_transformation<'a, T, F>(
    mut elements: impl Iterator<Item = &'a str>,
    statement: &str,
    entity: &mut Transformation,
    apply: F,
) -> AResult<String>
where
    T: ToString + From<(f32, f32, f32)> + Copy,
    F: FnOnce(&Transformation, T) -> Transformation,
{
    let transformation: T = (
        parse_coordinate(next_element!(elements, statement, 4), entity.translation.x)?,
        parse_coordinate(next_element!(elements, statement, 5), entity.translation.y)?,
        parse_coordinate(next_element!(elements, statement, 6), entity.translation.z)?,
    )
        .into();
    *entity = apply(entity, transformation);
    Ok(transformation.to_string())
}

const KEYWORDS: &[&str] = &[
    "move",
    "translate",
    "m",
    "turn",
    "rotate",
    "r",
    "size",
    "scale",
    "s",
    "end",
];
fn get_keyword(input: &str) -> Option<&str> {
    KEYWORDS.iter().find(|&&k| input.contains(k)).copied()
}

fn parse_coordinate(coordinate: &str, current: f32) -> AResult<f32> {
    let coordinate = coordinate.to_string();
    if coordinate.starts_with('~') {
        Ok(coordinate
            .replace("~-", "-0")
            .replace('~', "0")
            .parse::<f32>()
            .map_err(|err| CompileError::InvalidCoordinate(coordinate.clone(), err))?
            + current)
    } else {
        coordinate
            .parse::<f32>()
            .map_err(|err| CompileError::InvalidCoordinate(coordinate.clone(), err).into())
    }
}
 */
