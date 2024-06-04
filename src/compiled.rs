use itertools::Itertools;

use crate::statements::{Program, Statement};

#[allow(clippy::module_name_repetitions)]
pub struct CompiledFile {
    pub path: String,
    pub object_name: String,
    pub animation_name: String,
    pub contents: String,
}

pub fn program(program: Program, file_name: &str, file_path: &str) -> CompiledFile {
    let mut object_name: String = file_name.to_string();
    let mut animation_name: String = file_name.to_string();
    let mut delay = 0;
    let program_contents = program
        .statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::ObjectName(object, animation) => {
                object_name = object;
                animation_name = animation;
                None
            }
            Statement::Wait(duration) => {
                delay += duration;
                None
            }
            Statement::Empty => None,

            Statement::Translate(entity, translation, duration) => {
                let compiled_transformation = translation.compile();
                Some(transformation(
                    &object_name,
                    &animation_name,
                    entity.name(),
                    delay,
                    duration,
                    &compiled_transformation,
                ))
            }
            Statement::Rotate(entity, rotation, duration) => {
                let compiled_transformation = rotation.compile();
                Some(transformation(
                    &object_name,
                    &animation_name,
                    entity.name(),
                    delay,
                    duration,
                    &compiled_transformation,
                ))
            }
            Statement::Scale(entity, scale, duration) => {
                let compiled_transformation = scale.compile();
                Some(transformation(
                    &object_name,
                    &animation_name,
                    entity.name(),
                    delay,
                    duration,
                    &compiled_transformation,
                ))
            }
            Statement::Spawn {
                source,
                entity_type,
                new,
            } => Some(spawn(
                &object_name,
                &animation_name,
                delay,
                &entity_type,
                new.name(),
                source.name(),
            )),
            Statement::Item(entity, item_definition) => Some(item(
                &object_name,
                &animation_name,
                entity.name(),
                delay,
                &item_definition,
            )),
            Statement::Block(entity, block_state) => Some(block(
                &object_name,
                &animation_name,
                entity.name(),
                delay,
                &block_state.compile(),
            )),
            Statement::Text(entity, text_string) => Some(text(
                &object_name,
                &animation_name,
                entity.name(),
                delay,
                &text_string,
            )),
        })
        .join("\n");

    CompiledFile {
        path: file_path.to_string(),
        object_name: object_name.to_string(),
        animation_name: animation_name.to_string(),
        contents: format!(
            "{}\n{}\n{}\n{}",
            disclaimer(),
            program_contents,
            reset(&object_name, &animation_name, delay),
            increment(&object_name, &animation_name),
        ),
    }
}

fn item(
    object_name: &str,
    animation_name: &str,
    entity_name: &str,
    delay: u32,
    item: &str,
) -> String {
    format!(
        "execute as @e[tag={object_name},tag={entity_name}] \
        if score ${object_name}-{animation_name} timer matches {delay} run \
        item replace entity @s contents with {item}"
    )
}

fn block(
    object_name: &str,
    animation_name: &str,
    entity_name: &str,
    delay: u32,
    block_state: &str,
) -> String {
    format!(
        "execute as @e[tag={object_name},tag={entity_name}] \
        if score ${object_name}-{animation_name} timer matches {delay} run \
        data merge entity @s {{block_state:{{{block_state}}}}}"
    )
}

fn text(
    object_name: &str,
    animation_name: &str,
    entity_name: &str,
    delay: u32,
    text: &str,
) -> String {
    format!(
        "execute as @e[tag={object_name},tag={entity_name}] \
        if score ${object_name}-{animation_name} timer matches {delay} run \
        data merge entity @s {{text:'{text}'}}"
    )
}

fn transformation(
    object_name: &str,
    animation_name: &str,
    entity_name: &str,
    delay: u32,
    duration: u32,
    transformation: &str,
) -> String {
    format!(
        "execute as @e[tag={object_name},tag={entity_name}] \
        if score ${object_name}-{animation_name} timer matches {delay} run \
        data merge entity @s {{start_interpolation:0,interpolation_duration:{duration},transformation:{{{transformation}}}}}"
    )
}

fn spawn(
    object_name: &str,
    animation_name: &str,
    delay: u32,
    entity_type: &str,
    new_entity_name: &str,
    source_entity_name: &str,
) -> String {
    format!(
        "execute as @e[tag={object_name},tag={source_entity_name}] at @s \
        if score ${object_name}-{animation_name} timer matches {delay} run \
        summon {entity_type} ~ ~ ~ {{Tags:[\"{object_name}\",\"{new_entity_name}\"]}}"
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
    "# File generated using DiSPA".to_string()
}
