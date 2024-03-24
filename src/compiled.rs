use itertools::Itertools;

use crate::{
    objects::NumberSet,
    statements::{Program, Statement, TransformStatement},
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
            Statement::Transform(numbers, entity, keyword) => {
                let compiled_transformation = match keyword {
                    TransformStatement::Translate(t) => t.compile(),
                    TransformStatement::Rotate(r) => r.compile(),
                    TransformStatement::Scale(s) => s.compile(),
                };
                Some(transformation(
                    &object_name,
                    &animation_name,
                    entity.name(),
                    numbers,
                    &compiled_transformation,
                ))
            }
            Statement::End(delay) => Some(reset(&object_name, &animation_name, delay)),
            Statement::Spawn {
                delay,
                source,
                entity_type,
                new,
                offset,
            } => Some(spawn(
                &object_name,
                &animation_name,
                delay,
                &entity_type,
                new.name(),
                source.name(),
                offset,
            )),
            Statement::Block(_) | Statement::BlockEnd | Statement::EndOfFile => None,
        })
        .join("\n");

    CompiledFile {
        path: file_path.to_string(),
        object_name: object_name.to_string(),
        animation_name: animation_name.to_string(),
        contents: disclaimer() + &program_contents + &increment(&object_name, &animation_name),
    }
}

fn transformation(
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

fn spawn(
    object_name: &str,
    animation_name: &str,
    delay: u32,
    entity_type: &str,
    new_entity_name: &str,
    source_entity_name: &str,
    offset: (f32, f32, f32),
) -> String {
    let (x, y, z) = offset;
    format!(
        "execute as @e[tag={object_name}-{source_entity_name}] at @s \
        if score ${object_name}-{animation_name} timer matches {delay} run \
        summon {entity_type} ~{x}, ~{y}, ~{z} {{Tags:[\"{object_name}-{new_entity_name}\"]}}"
    )
}

fn reset(object_name: &str, animation_name: &str, delay: u32) -> String {
    format!(
        "\n\
        execute if score ${object_name}-{animation_name} timer matches {delay}.. run scoreboard players set ${object_name}-{animation_name} flags 0\n\
        execute if score ${object_name}-{animation_name} timer matches {delay}.. run scoreboard players set ${object_name}-{animation_name} timer -1\n\
        "
    )
}

fn increment(object_name: &str, animation_name: &str) -> String {
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
