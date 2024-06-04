use std::{
    fmt::Display,
    fs,
    io::{stdin, Read, Write},
    path::PathBuf,
    str::FromStr,
};

use anyhow::ensure;
use file_reader::parse_file;
use walkdir::WalkDir;

use crate::errors::GenericError;

mod compiled;
mod config;
mod errors;
mod file_reader;
mod objects;
mod statements;

fn get_folder_tree(path: PathBuf) -> Vec<String> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|path| {
            if let Err(ref err) = path {
                println!("{}", GenericError::InvalidPath(err.to_string()));
            }
            path.ok()
        })
        .filter(|path| {
            path.path()
                .extension()
                .is_some_and(|e| e == DISPA_EXTENSION)
        })
        .filter_map(|path| path.into_path().into_os_string().into_string().ok())
        .collect::<Vec<_>>()
}

/// Collects all the 'Ok' values in the input and flattens the Results into the output.
///
/// # Errors
/// If any of the results in the input are Err, this returns a `GenericError::Collection` containing all of the errors.
pub fn collect_errors<T, E: Display>(input: Vec<Result<T, E>>) -> anyhow::Result<Vec<T>> {
    let errors = input
        .iter()
        .enumerate()
        .filter_map(|(index, element)| element.as_ref().err().map(|e| (index, e)))
        .fold(String::new(), |mut acc, err| {
            acc.push_str(&format!("{}: {}\n", err.0, err.1));
            acc
        });
    ensure!(errors.is_empty(), GenericError::Collection(errors));
    Ok(input.into_iter().filter_map(Result::ok).collect())
}

const DISPA_EXTENSION: &str = "dspa";
const MINECRAFT_EXTENSION: &str = "mcfunction";

fn main() -> anyhow::Result<()> {
    let config = config::read()?;
    let files = get_folder_tree(PathBuf::from_str(&config.source_folder).unwrap());
    let results = files.into_iter().map(|path| parse_file(&path)).collect();
    fs::write(&config.tick_function, "")?;
    for result in collect_errors(results)? {
        let path: String = result
            .path
            .replace(&config.source_folder, &config.target_folder)
            .replace(DISPA_EXTENSION, MINECRAFT_EXTENSION);
        fs::write(&path, result.contents)?;
        let filtered_path = path
            .replace('\\', "/")
            .strip_prefix("./")
            .unwrap_or(&path)
            .replace(&format!(".{MINECRAFT_EXTENSION}"), "");
        let mut tick_function = fs::OpenOptions::new()
            .append(true)
            .open(&config.tick_function)?;
        writeln!(
            tick_function,
            "{}",
            compiled::tick_function_line(
                &result.object_name,
                &result.animation_name,
                &config.namespace,
                &filtered_path
            ),
        )?;
        println!("Successfully Compiled file: {filtered_path}");
    }

    println!("Press Enter to continue...");
    let _ = std::io::stdout().flush();
    let _ = stdin().read(&mut [0_u8]);
    Ok(())
}
