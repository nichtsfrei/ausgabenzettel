// src/git.rs

use std::path::Path;
use std::process::Command;

use tokio::task::spawn_blocking;

pub async fn is_git_repo(path: &Path) -> bool {
    let path_owned = path.to_path_buf();
    spawn_blocking(move || {
        Command::new("git")
            .arg("-C")
            .arg(&path_owned)
            .arg("rev-parse")
            .arg("--is-inside-work-tree")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

pub async fn git_commit(
    path: &Path,
    filename: String,
    sha256: String,
) -> Result<(), String> {
    let path_owned = path.to_path_buf();
    let result = spawn_blocking(move || {
        let output = match Command::new("git")
            .arg("-C")
            .arg(&path_owned)
            .arg("add")
            .arg("-A")
            .output()
        {
            Ok(o) => o,
            Err(e) => return Err(format!("git add failed: {}", e)),
        };

        if !output.status.success() {
            return Err(format!("git add failed: {}", output.status));
        }

        let message = format!("Auto: user upload: {filename} (sha256: {sha256})");

        let output = match Command::new("git")
            .arg("-C")
            .arg(&path_owned)
            .arg("commit")
            .arg("-m")
            .arg(&message)
            .output()
        {
            Ok(o) => o,
            Err(e) => return Err(format!("git commit failed: {}", e)),
        };

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("git commit failed: {}", output.status))
        }
    })
    .await;

    match result {
        Ok(r) => r,
        Err(e) => Err(format!("Git task failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_git_repo() {
        assert!(!is_git_repo(std::path::Path::new("/tmp")).await);
    }
}
