use itertools::Itertools;

use crate::statements::{Program, Statement};

#[allow(clippy::module_name_repetitions)]
pub struct CompiledFile {
    pub path: String,
    pub object_name: String,
    pub animation_name: String,
    pub contents: String,
}
struct ProgramData {
    object_name: String,
    animation_name: String,
    delay: u32,
}
impl ProgramData {
    fn new(file_name: &str) -> Self {
        Self {
            object_name: file_name.to_string(),
            animation_name: file_name.to_string(),
            delay: 0,
        }
    }
    #[allow(clippy::needless_pass_by_value)]
    fn execute_string(&self, entity_name: &str, command: String) -> String {
        format!(
            "execute as @e[tag={0},tag={entity_name}] if score ${0}-{1} timer matches {2} run {command}",
            self.object_name, self.animation_name, self.delay
        )
    }
    #[allow(clippy::needless_pass_by_value)]
    fn execute_at_string(&self, entity_name: &str, command: String) -> String {
        format!(
            "execute as @e[tag={0},tag={entity_name}] at @s if score ${0}-{1} timer matches {2} run {command}",
            self.object_name, self.animation_name, self.delay
        )
    }
}
pub fn program(program: Program, file_name: &str, file_path: &str) -> CompiledFile {
    let mut data = ProgramData::new(file_name);
    let program_contents = program
        .statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::ObjectName(object, animation) => {
                data.object_name = object;
                data.animation_name = animation;
                None
            }
            Statement::Wait(duration) => {
                data.delay += duration;
                None
            }
            Statement::Empty => None,

            Statement::Translate(entity, translation, duration) => {
                let compiled_transformation = translation.compile();
                Some(transformation(
                    &data,
                    entity.name(),
                    duration,
                    &compiled_transformation,
                ))
            }
            Statement::Rotate(entity, rotation, duration) => Some(transformation(
                &data,
                entity.name(),
                duration,
                &rotation.compile(),
            )),
            Statement::Scale(entity, scale, duration) => Some(transformation(
                &data,
                entity.name(),
                duration,
                &scale.compile(),
            )),
            Statement::Spawn(source, entity_type, new) => {
                Some(spawn(&data, &entity_type, new.name(), source.name()))
            }
            Statement::Item(entity, item_definition) => {
                Some(item(&data, entity.name(), &item_definition))
            }
            Statement::Block(entity, block_state) => {
                Some(block(&data, entity.name(), &block_state.compile()))
            }
            Statement::Text(entity, text_string) => Some(text(&data, entity.name(), &text_string)),
            Statement::Teleport(entity, x, y, z) => Some(teleport(&data, entity.name(), x, y, z)),
            Statement::Raw(command, delayed) => Some(raw(&data, &command, delayed)),
        })
        .join("\n");

    CompiledFile {
        path: file_path.to_string(),
        object_name: data.object_name.to_string(),
        animation_name: data.animation_name.to_string(),
        contents: format!(
            "{}\n{}\n{}\n{}",
            disclaimer(),
            program_contents,
            reset(&data),
            increment(&data),
        ),
    }
}

pub fn disclaimer() -> String {
    "# File generated using DiSPA".to_string()
}

pub fn tick_function_line(
    object_name: &str,
    animation_name: &str,
    namespace: &str,
    path: &str,
) -> String {
    format!("execute if score ${object_name}-{animation_name} flags matches 1.. run function {namespace}:{path}")
}

fn increment(data: &ProgramData) -> String {
    let object_name = &data.object_name;
    let animation_name = &data.animation_name;
    format!("scoreboard players add ${object_name}-{animation_name} timer 1")
}

fn reset(data: &ProgramData) -> String {
    let ProgramData {
        object_name,
        animation_name,
        delay,
    } = data;
    format!(
        "\n\
        execute if score ${object_name}-{animation_name} timer matches {delay}.. run scoreboard players set ${object_name}-{animation_name} flags 0\n\
        execute if score ${object_name}-{animation_name} timer matches {delay}.. run scoreboard players set ${object_name}-{animation_name} timer -1\n\
        "
    )
}

fn raw(data: &ProgramData, command: &str, delayed: bool) -> String {
    if delayed {
        format!(
            "execute if score ${0}-{1} timer matches {2} run {3}",
            data.object_name, data.animation_name, data.delay, command
        )
    } else {
        command.to_string()
    }
}

fn transformation(
    data: &ProgramData,
    entity_name: &str,
    duration: u32,
    transformation: &str,
) -> String {
    data.execute_string(
        entity_name,
        format!("data merge entity @s {{start_interpolation:0,interpolation_duration:{duration},transformation:{{{transformation}}}}}")
    )
}

fn spawn(
    data: &ProgramData,
    entity_type: &str,
    new_entity_name: &str,
    source_entity_name: &str,
) -> String {
    data.execute_at_string(
        source_entity_name,
        format!(
            "summon {entity_type} ~ ~ ~ {{Tags:[\"{}\",\"{new_entity_name}\"]}}",
            data.object_name
        ),
    )
}

fn item(data: &ProgramData, entity_name: &str, item: &str) -> String {
    data.execute_string(
        entity_name,
        format!("item replace entity @s contents with {item}"),
    )
}

fn block(data: &ProgramData, entity_name: &str, block_state: &str) -> String {
    data.execute_string(
        entity_name,
        format!("data merge entity @s {{block_state:{{{block_state}}}}}"),
    )
}

fn text(data: &ProgramData, entity_name: &str, text: &str) -> String {
    data.execute_string(
        entity_name,
        format!("data merge entity @s {{text:'{text}'}}"),
    )
}

fn teleport(data: &ProgramData, entity_name: &str, x: f32, y: f32, z: f32) -> String {
    data.execute_at_string(entity_name, format!("tp @s ~{x} ~{y} ~{z}"))
}
