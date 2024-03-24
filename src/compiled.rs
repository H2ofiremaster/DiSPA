use itertools::Itertools;

use crate::{
    objects::NumberSet,
    statements::{KeywordStatement, Program, Statement},
};

pub struct CompiledFile {
    pub path: String,
    pub object_name: String,
    pub animation_name: String,
    pub contents: String,
}

pub fn program(program: Program, file_name: &str, file_path: &str) -> CompiledFile {
    let mut object_name: String = file_name.to_string();
    let mut animation_name: String = file_name.to_string();
    let program_contents = program
        .statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::ObjectName(name) => {
                object_name = name;
                None
            }
            Statement::AnimationName(name) => {
                animation_name = name;
                None
            }
            Statement::Keyword(numbers, entity, keyword) => {
                let compiled_transformation = match keyword {
                    KeywordStatement::Translate(t) => t.compile(),
                    KeywordStatement::Rotate(r) => r.compile(),
                    KeywordStatement::Scale(s) => s.compile(),
                    KeywordStatement::Spawn(_name, _offset) => todo!(),
                };
                Some(transformation(
                    &object_name,
                    &animation_name,
                    &entity,
                    numbers,
                    &compiled_transformation,
                ))
            }
            Statement::End(delay) => Some(reset(&object_name, &animation_name, delay)),
            _ => None,
        })
        .join("\n");

    CompiledFile {
        path: file_path.to_string(),
        object_name: object_name.to_string(),
        animation_name: animation_name.to_string(),
        contents: disclaimer() + &program_contents + &increment(&object_name, &animation_name),
    }
}

pub fn transformation(
    object_name: &str,
    animation_name: &str,
    entity_name: &str,
    values: NumberSet,
    transformation: &str,
) -> String {
    let NumberSet { delay, duration } = values;
    format!(
        "execute as @e[tag={object_name}-{entity_name}] \
        if score ${object_name}-{animation_name} timer matches {delay} run \
        data merge entity @s {{start_interpolation:0,interpolation_duration:{duration},transformation: {{{transformation}}}}}"
    )
}

pub fn reset(object_name: &str, animation_name: &str, delay: u32) -> String {
    format!(
        "\n\
        execute if score ${object_name}-{animation_name} timer matches {delay}.. run scoreboard players set ${object_name}-{animation_name} flags 0\n\
        execute if score ${object_name}-{animation_name} timer matches {delay}.. run scoreboard players set ${object_name}-{animation_name} timer -1\n\
        "
    )
}

pub fn increment(object_name: &str, animation_name: &str) -> String {
    format!("scoreboard players add ${object_name}-{animation_name} timer 1")
}

pub fn tick_function_line(
    object_name: &str,
    animation_name: &str,
    namespace: &str,
    path: &str,
) -> String {
    format!("execute if score ${object_name}-{animation_name} flags matches 1.. run function {namespace}:{path}")
}

pub fn disclaimer() -> String {
    "# File generated using DiSPA\n".to_string()
}
