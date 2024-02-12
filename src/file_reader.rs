use anyhow::{bail, Context};
use anyhow::{ensure, Result as AResult};
use regex::Regex;
use std::collections::HashMap;
use std::fs::read_to_string;

use crate::objects::{Rotation, Scale, Transformation, Translation};
use crate::{collect_errors, compiled};

macro_rules! next_element {
    ($elements:expr, $statement:expr, $current:expr) => {
        $elements.next().with_context(|| {
            format!(
                "Statement '{}' has too few elements: {}/7",
                $statement.escape_default(),
                $current,
            )
        })?
    };
}

#[derive(Debug, Default)]
struct Block {
    delay: u32,
    duration: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum NumberType {
    Delay,
    Duration,
}
impl TryFrom<char> for NumberType {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            't' => Ok(Self::Delay),
            'd' => Ok(Self::Duration),
            _ => bail!("Char '{value}' does not correspond to a NumberType."),
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct Number {
    number_type: NumberType,
    value: u32,
}
impl TryFrom<&str> for Number {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split = value.split_once('_').with_context(|| {
            format!("Number {value} did not contain type discriminator: No underscore.")
        })?;
        let number_type = NumberType::try_from(split.1.chars().next().with_context(|| {
            format!("Number {value} did not contain type discriminator: No discriminator.")
        })?)?;
        let value: u32 = split.0.parse()?;
        Ok(Number { number_type, value })
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
    let comment_regex = Regex::new(COMMENT_PATTERN)
        .with_context(|| format!("Pattern '{}' is not a valid regex", COMMENT_PATTERN))?;

    let file_name = extract_name(path)?;

    let raw_code = read_to_string(path).context("Path does not lead to a valid file.")?;
    let filtered_code = comment_regex.replace_all(&raw_code, "");

    let statements: Vec<_> = filtered_code
        .split(';')
        .flat_map(|s| s.split_inclusive(&['{', '}']))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    ensure!(!statements.is_empty(), "File is empty.");
    let (object_name, animation_name, statements) = parse_names(statements, file_name)?;
    let flattened = collapse_blocks(statements)?;
    let mut compiled = compile_statements(flattened, &object_name, &animation_name)?;
    compiled.push(format!(
        "scoreboard players add ${object_name}-{animation_name} timer 1"
    ));

    Ok(CompiledFile {
        path: path.to_string(),
        object_name,
        animation_name,
        contents: String::from("# File generated using DiSPA\n") + &compiled.join("\n"),
    })
}

const NAME_PATTERN: &str = r"^[A-Za-z0-9_\-]*$";
fn parse_names(
    statements: Vec<String>,
    file_name: String,
) -> AResult<(String, String, Vec<String>)> {
    let name_regex = Regex::new(NAME_PATTERN)
        .with_context(|| format!("Pattern '{}' is not a valid regex", COMMENT_PATTERN))?;

    let object_name = statements
        .iter()
        .find(|&statement| statement.starts_with("object"))
        .map(|s| {
            s.split_once(' ')
                .context("Object name specified, but not provided.")
                .map(|r| r.1.to_string())
        });
    let object_name = match object_name {
        Some(Ok(value)) => value,
        Some(Err(e)) => return Err(e),
        None => file_name,
    };

    ensure!(
        name_regex.is_match(&object_name),
        "Object name contains invalid characters."
    );

    let animation_name = statements
        .iter()
        .find(|&statement| statement.starts_with("anim"))
        .map(|s| {
            s.split_once(' ')
                .context("Animation name specified, but not provided.")
                .map(|r| r.1.to_string())
        });
    let animation_name = match animation_name {
        Some(Ok(value)) => value,
        Some(Err(e)) => return Err(e),
        None => object_name.clone(),
    };
    ensure!(
        name_regex.is_match(&animation_name),
        "Animation name contains invalid characters."
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

fn extract_name(path: &str) -> AResult<String> {
    let mut split = path
        .split(['\\', '/'])
        .last()
        .context("Path was empty.")?
        .split('.')
        .collect::<Vec<_>>();
    split.pop();
    Ok(split.join("."))
}

fn collapse_blocks(statements: impl IntoIterator<Item = String>) -> AResult<Vec<String>> {
    let mut blocks: Vec<Block> = vec![Block::default()];
    let flattened: Vec<anyhow::Result<String>> = statements
        .into_iter()
        .map(|statement| {
            if statement.contains('{') {
                let previous_block = blocks.last().context("Block queue is empty.")?;
                let (delay, duration) = parse_block_definition(&statement, previous_block)?;
                blocks.push(Block { delay, duration });
                return Ok(String::new());
            } else if statement == "}" {
                if blocks.len() > 1 {
                    blocks.pop();
                    return Ok(String::new());
                } else {
                    bail!("Unbalanced brackets: Too many closing brackets.")
                }
            }

            let current_block = blocks.last().context("Block queue is empty.")?;
            let keyword = get_keyword(&statement).with_context(|| {
                format!(
                    "Statement '{}' doesn't contain a keyword.",
                    statement.escape_default()
                )
            })?;
            let statement_start = statement.split_inclusive(keyword).next().with_context(|| {
                format!(
                    "Statement '{}' doesn't contain a keyword.",
                    statement.escape_default()
                )
            })?;
            let mut statement = statement.clone();
            if !statement_start.contains("_d") {
                statement = format!("{}_d {}", current_block.duration, statement);
            }
            if !statement_start.contains("_t") {
                statement = format!("{}_t {}", current_block.delay, statement);
            }

            Ok(statement)
        })
        .filter(|statement| !matches!(statement, Ok(statement) if statement.is_empty()))
        .collect();
    collect_errors(flattened)
}

fn parse_block_definition(definition: &str, previous_block: &Block) -> AResult<(u32, u32)> {
    let numbers: Vec<AResult<Number>> = definition
        .split(' ')
        .filter(|&s| s != "{")
        .map(Number::try_from)
        .collect();
    let numbers = collect_errors(numbers)?;
    if numbers.len() > 2 {
        bail!("Too many numbers in block definition: {:?}", numbers)
    }
    let (first, second) = (
        numbers.get(0).context("Block definition is empty.")?,
        numbers.get(1),
    );
    if second.is_some_and(|second| second.number_type == first.number_type) {
        bail!("Block definition contains two numbers of the same type.")
    }
    Ok(numbers.iter().fold(
        (previous_block.delay, previous_block.duration),
        |(delay, duration), number| match number.number_type {
            NumberType::Delay => (number.value, duration),
            NumberType::Duration => (delay, number.value),
        },
    ))
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
            let numbers: (Number, Number) = (
                next_element!(elements, statement, 0).try_into()?,
                next_element!(elements, statement, 1).try_into()?,
            );
            let (delay, duration) = order_numbers(numbers)?;
            let keyword = next_element!(elements, statement, 2);
            if keyword == "end" {
                println!("Delay: {delay}, Duration: {duration}");
                return Ok(compiled::reset(object_name, animation_name, delay));
            }
            let entity_name = next_element!(elements, statement, 3).to_owned();
            let entity = entities.get(&entity_name);
            let entity = match entity {
                Some(entity) => entity,
                None => {
                    entities.insert(entity_name.clone(), Transformation::default());
                    entities.get(&entity_name).unwrap()
                }
            };
            let transformation = match keyword {
                "move" | "translate" | "m" => {
                    let translation = Translation {
                        x: parse_coordinate(
                            next_element!(elements, statement, 4),
                            entity.translation.x,
                        )?,
                        y: parse_coordinate(
                            next_element!(elements, statement, 5),
                            entity.translation.y,
                        )?,
                        z: parse_coordinate(
                            next_element!(elements, statement, 6),
                            entity.translation.z,
                        )?,
                    };
                    entities.insert(entity_name.clone(), entity.with_translation(translation));
                    translation.to_string()
                }
                "turn" | "rotatie" | "r" => {
                    let rotation = Rotation {
                        yaw: parse_coordinate(
                            next_element!(elements, statement, 4),
                            entity.rotation.yaw,
                        )?,
                        pitch: parse_coordinate(
                            next_element!(elements, statement, 5),
                            entity.rotation.pitch,
                        )?,
                        roll: parse_coordinate(
                            next_element!(elements, statement, 6),
                            entity.rotation.roll,
                        )?,
                    };
                    entities.insert(entity_name.clone(), entity.with_rotation(rotation));
                    rotation.to_string()
                }
                "size" | "scale" | "s" => {
                    let scale = Scale {
                        x: parse_coordinate(next_element!(elements, statement, 4), entity.scale.x)?,
                        y: parse_coordinate(next_element!(elements, statement, 5), entity.scale.y)?,
                        z: parse_coordinate(next_element!(elements, statement, 6), entity.scale.z)?,
                    };
                    entities.insert(entity_name.clone(), entity.with_scale(scale));
                    scale.to_string()
                }
                _ => bail!(
                    "Statement '{}' has an invalid keyword.",
                    statement.escape_default()
                ),
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

fn order_numbers(numbers: (Number, Number)) -> AResult<(u32, u32)> {
    Ok((
        match (numbers.0.number_type, numbers.1.number_type) {
            (NumberType::Delay, _) => numbers.0.value,
            (_, NumberType::Delay) => numbers.1.value,
            _ => bail!("Statement does not contain Delay."),
        },
        match (numbers.0.number_type, numbers.1.number_type) {
            (NumberType::Duration, _) => numbers.0.value,
            (_, NumberType::Duration) => numbers.1.value,
            _ => bail!("Statement does not contain Duration."),
        },
    ))
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
    let coordinate = coordinate.to_owned();
    if coordinate.starts_with('~') {
        Ok(coordinate
            .replace("~-", "-0")
            .replace('~', "0")
            .parse::<f32>()
            .with_context(|| {
                format!(
                    "Could not parse coordinate '{}'.",
                    coordinate.escape_default()
                )
            })?
            + current)
    } else {
        coordinate.parse::<f32>().with_context(|| {
            format!(
                "Could not parse coordinate '{}'.",
                coordinate.escape_default()
            )
        })
    }
}
