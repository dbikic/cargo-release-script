use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use cargo_metadata::MetadataCommand;
use error_chain::error_chain;
use semver::{BuildMetadata, Prerelease, Version};

use crate::BumpType;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Semver(semver::Error);
    }
}

pub fn increment_version(version_change: BumpType) -> Result<String> {
    let current_version = get_current_version();
    let mut parsed_version = Version::parse(&current_version)?;
    match version_change {
        BumpType::MAJOR => increment_major(&mut parsed_version),
        BumpType::MINOR => increment_minor(&mut parsed_version),
        BumpType::PATCH => increment_patch(&mut parsed_version),
    }
    let version = update_version(parsed_version.to_string())?;
    update_chart_version_if_available(parsed_version.to_string())?;
    Ok(version)
}

fn get_current_version() -> String {
    let meta = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .unwrap();
    let root = meta.root_package().unwrap();
    root.version.to_string()
}

fn update_version(new_version: String) -> std::io::Result<String> {
    let file_path = "Cargo.toml";
    let read_file = OpenOptions::new().read(true).open(file_path)?;
    let buffered = BufReader::new(read_file);
    let mut result: Vec<String> = vec![];
    for (i, line) in buffered.lines().enumerate() {
        let new_line = line.unwrap().clone();
        if i == 2 {
            result.push(format!("version = \"{}\"\n", new_version));
        } else {
            result.push(format!("{}\n", new_line));
        }
    }
    write_lines_into_file(file_path.to_string(), result)?;
    Ok(new_version)
}

fn update_chart_version_if_available(new_version: String) -> std::io::Result<()> {
    let file_path = ".deployment/Chart.yaml";
    if Path::new(file_path).exists() {
        let read_file = OpenOptions::new().read(true).open(file_path)?;
        let buffered = BufReader::new(read_file);
        let mut result: Vec<String> = vec![];
        for (i, line) in buffered.lines().enumerate() {
            let new_line = line.unwrap().clone();
            if i == 20 {
                result.push(format!("appVersion: {}\n", new_version));
            } else {
                result.push(format!("{}\n", new_line));
            }
        }
        write_lines_into_file(file_path.to_string(), result)?;
    }
    Ok(())
}

fn write_lines_into_file(file_path: String, lines: Vec<String>) -> std::io::Result<()> {
    let mut write_file = OpenOptions::new().write(true).open(file_path)?;
    for x in lines.iter().map(|x| x.as_bytes()) {
        write_file.write_all(x)?;
    }
    Ok(())
}

fn increment_patch(v: &mut Version) {
    v.patch += 1;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_minor(v: &mut Version) {
    v.minor += 1;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_major(v: &mut Version) {
    v.major += 1;
    v.minor = 0;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}
