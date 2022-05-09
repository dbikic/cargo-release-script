use std::fmt::Debug;
use std::process::Command;

use error_chain::error_chain;

error_chain! {
    foreign_links {
        Io(std::io::Error);
    }
}

pub fn check_if_on_correct_branch() -> Result<String> {
    let command = Command::new("sh")
        .arg("-c")
        .arg("git rev-parse --abbrev-ref HEAD")
        .output()
        .expect("failed to execute process");
    let current_branch = String::from_utf8(command.stdout)
        .unwrap()
        .trim()
        .to_string();
    match current_branch.as_str() {
        "main" => Ok("main".to_string()),
        "master" => Ok("master".to_string()),
        _ => panic!("You need to be on the main or master branch to run the release script!"),
    }
}

pub fn commit_version_change(version: &str, origin_branch: String) -> Result<()> {
    execute_command(
        format!("git pull origin {}", origin_branch),
        format!(
            "Failed to pull branch: {}!",
            origin_branch
        ),
    )?;
    let branch_name = format!("release-{}", version);
    execute_command(
        format!("git checkout -b {}", branch_name),
        format!(
            "Failed to crate branch: {}! Maybe it already exists?",
            branch_name
        ),
    )?;
    execute_command(
        "cargo build".to_string(),
        "Failed to compile the project!".to_string(),
    )?;
    execute_command(
        "git add -A".to_string(),
        "Failed to execute git add".to_string(),
    )?;
    execute_command(
        format!("git commit -m \"v{}\"", version),
        "Failed to create commit".to_string(),
    )?;
    execute_command(
        format!("git push origin {}", branch_name),
        "Failed to create commit".to_string(),
    )?;
    execute_command(
        format!("git checkout {}", origin_branch),
        format!("Failed to checkout: {}", origin_branch),
    )?;
    execute_command(
        format!("git branch -D {}", branch_name),
        format!("Failed to delete local release branch: {}", branch_name),
    )?;
    Ok(())
}

fn execute_command(command: String, error_message: String) -> Result<()> {
    let command = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect(&error_message);
    if command.status.code().unwrap() != 0 {
        panic!("{}", error_message);
    } else {
        Ok(())
    }
}

pub fn get_repo_name() -> Result<(String, String)> {
    let command = Command::new("sh")
        .arg("-c")
        .arg("git config --get remote.origin.url
")
        .output()
        .expect("failed to execute process");
    let origin_url = String::from_utf8(command.stdout)
        .unwrap()
        .trim()
        .to_string();
    let mut start = origin_url.rfind('/').unwrap();
    let repo_name = &origin_url[start + 1..].strip_suffix(".git").unwrap();
    start = origin_url.find(':').unwrap();
    let end = origin_url.rfind('/').unwrap();
    let repo_owner = &origin_url[start + 1..end];
    Ok((repo_owner.to_string(), repo_name.to_string()))
}
