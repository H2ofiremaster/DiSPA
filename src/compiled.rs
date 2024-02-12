pub fn transformation(
    object_name: &str,
    animation_name: &str,
    entity_name: &str,
    delay: u32,
    duration: u32,
    transformation: &str,
) -> String {
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

pub fn increment(object_name: &str, animation_name: &str, namespace: &str, path: &str) -> String {
    format!("execute if score ${object_name}-{animation_name} flags matches 1.. run function {namespace}:{path}")
}
