use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub source_folder: String,
    pub target_folder: String,
    pub tick_function: String,
    pub namespace: String,
}

const CONFIG_PATH: &str = "./dspa_config.json";
pub fn read_config() -> anyhow::Result<Config> {
    let config_contents = match fs::read_to_string(CONFIG_PATH) {
        Ok(contents) => contents,
        Err(_) => initialize_config_file(),
    };
    Ok(serde_json::from_str::<Config>(&config_contents)?)
}

const CONFIG_DEFAULTS: &str = r#"
{
    "source_folder": "./src",
    "target_folder": "./objects",
    "tick_function": "./tick.mcfunction",
    "namespace": "de"
}
"#;
fn initialize_config_file() -> String {
    fs::write("./dspa_config.json", CONFIG_DEFAULTS).expect("config path should be valid.");
    CONFIG_DEFAULTS.into()
}
