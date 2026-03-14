// src/git.rs

use std::path::Path;
use std::process::Command;

pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn git_commit(
    path: &Path,
    filename: String,
    sha256: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("add")
        .arg("-A")
        .output()?;

    if !output.status.success() {
        return Err(format!("git add failed: {}", output.status).into());
    }

    let message = format!("Auto: user upload: {filename} (sha256: {sha256})");

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("git commit failed: {}", output.status).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_repo() {
        assert!(!is_git_repo(std::path::Path::new("/tmp")));
    }
}
