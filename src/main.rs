use clap::{Arg, Command};
use error_chain::error_chain;

use crate::git_manager::{check_if_on_correct_branch, commit_version_change, get_repo_name};
use crate::version_manager::increment_version;

mod git_manager;
mod version_manager;

// 1) Parse is major, minor or patch change
// 2) Read the current project version
// 3) Increment the version in a correct way
// 3) Overwrite the project version
// 4) Commit the version change
// 5) Open the PR by clicking the link in the terminal
// 6) After merging the PR, a tag will be automatically created and pushed, which will release the
// app to production

error_chain! {
    foreign_links {
        VersionManagerError(version_manager::Error);
        GitManagerError(git_manager::Error);
    }
}

fn main() -> Result<()> {
    let matches = Command::new("Release script")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Dino Bikic <bikic.dino@gmail.com>")
        .arg(
            Arg::new("bump_type")
                .short('b')
                .long("bump_type")
                .takes_value(true)
                .help("Bump type. Needs to be one of the following: major, minor, patch"),
        )
        .get_matches();
    let bump_type_input: &str = matches
        .value_of("bump_type")
        .unwrap_or_else(|| panic!("Bump type not provided!"));
    let bump_type = BumpType::from(bump_type_input);
    let branch = check_if_on_correct_branch()?;
    let version = increment_version(bump_type)?;
    commit_version_change(&version, branch)?;
    println!("Version bumped to: {}", version);
    println!(
        "Now just open and merge the PR from the link below, and the production will be released!"
    );
    println!();
    let (repo_owner, repo_name) = get_repo_name()?;
    println!(
        "https://github.com/{}/{}/compare/release-{}?expand=1",
        repo_owner, repo_name, version,
    );
    Ok(())
}

pub enum BumpType {
    MAJOR,
    MINOR,
    PATCH,
}

impl From<&str> for BumpType {
    fn from(input: &str) -> Self {
        match input {
            "major" => BumpType::MAJOR,
            "minor" => BumpType::MINOR,
            "patch" => BumpType::PATCH,
            _ => panic!("Bump type needs to be one of the following: major, minor, patch"),
        }
    }
}
