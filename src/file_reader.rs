use std::{fs, path::Path};

use crate::{
    compiled::{self, CompiledFile},
    errors::GenericError,
    objects::TrackedChar,
    statements::{FileInfo, Program},
};

pub fn parse_file(file_path: &str) -> anyhow::Result<CompiledFile> {
    let contents = fs::read_to_string(file_path)
        .map_err(|err| GenericError::InvalidPath(err.to_string()))?
        .replace('\r', "");
    let chars = to_tracked(&contents);
    let program = Program::parse_from_file(
        &FileInfo::new(
            file_path.to_string(),
            TrackedChar::new(
                contents.chars().filter(|&c| c == '\n').count(),
                contents.lines().last().map_or(0, str::len),
                contents.chars().last().unwrap_or('\n'),
            ),
        ),
        &chars,
    );

    // println!("{program:#?}");
    Ok(compiled::program(
        program?,
        &get_file_name(file_path)?,
        file_path,
    ))
}

pub fn to_tracked(string: &str) -> Vec<TrackedChar> {
    string
        .split_inclusive('\n')
        .enumerate()
        .flat_map(|(line_number, line)| {
            line.chars()
                .enumerate()
                .map(move |(column_number, character)| {
                    TrackedChar::new(line_number + 1, column_number + 1, character)
                })
        })
        .collect()
}

fn get_file_name(path: &str) -> anyhow::Result<String> {
    let file_name = Path::new(path)
        .file_stem()
        .ok_or_else(|| GenericError::FileNotExist(path.to_string()))?
        .to_string_lossy();
    Ok(file_name.into_owned())
}
