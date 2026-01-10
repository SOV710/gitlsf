//! Git command interaction module.
//!
//! This module provides functions for interacting with Git repositories,
//! primarily through the `git ls-files` command to retrieve tracked files.

use std::path::Path;
use std::process::Command;

use crate::error::{GitlsError, Result};

/// Retrieves the list of files tracked by Git in the specified directory.
///
/// Uses `git ls-files` to get all tracked files in the repository.
///
/// # Arguments
///
/// * `path` - The path to the Git repository (or a subdirectory within it).
///
/// # Returns
///
/// A vector of file paths relative to the repository root.
///
/// # Errors
///
/// Returns an error if:
/// - The path is not within a Git repository
/// - The `git` command fails to execute
/// - The output cannot be parsed as UTF-8
///
/// # Example
///
/// ```no_run
/// use gitls::git::list_files;
///
/// let files = list_files(".").unwrap();
/// for file in files {
///     println!("{}", file);
/// }
/// ```
pub fn list_files(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let path = path.as_ref();

    let output = Command::new("git")
        .arg("ls-files")
        .current_dir(path)
        .output()
        .map_err(|e| GitlsError::git_with_source("Failed to execute git ls-files", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitlsError::NotAGitRepository);
        }
        return Err(GitlsError::git(format!(
            "git ls-files failed: {}",
            stderr.trim()
        )));
    }

    let stdout =
        String::from_utf8(output.stdout).map_err(|e| GitlsError::utf8("git ls-files output", e))?;

    let files: Vec<String> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    Ok(files)
}

/// Checks if a directory is within a Git repository.
///
/// # Arguments
///
/// * `path` - The path to check.
///
/// # Returns
///
/// `true` if the path is within a Git repository, `false` otherwise.
///
/// # Example
///
/// ```no_run
/// use gitls::git::is_git_repository;
///
/// if is_git_repository(".") {
///     println!("This is a Git repository!");
/// }
/// ```
pub fn is_git_repository(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_git_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();

        // Configure git user for commits
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(path)
            .output()
            .unwrap();

        // Create some test files
        fs::write(path.join("file1.rs"), "fn main() {}\n").unwrap();
        fs::write(path.join("file2.txt"), "hello\nworld\n").unwrap();

        // Add files to git
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_list_files_in_git_repo() {
        let temp_dir = setup_git_repo();
        let files = list_files(temp_dir.path()).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1.rs".to_string()));
        assert!(files.contains(&"file2.txt".to_string()));
    }

    #[test]
    fn test_list_files_not_a_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = list_files(temp_dir.path());

        assert!(result.is_err());
        matches!(result.unwrap_err(), GitlsError::NotAGitRepository);
    }

    #[test]
    fn test_is_git_repository_true() {
        let temp_dir = setup_git_repo();
        assert!(is_git_repository(temp_dir.path()));
    }

    #[test]
    fn test_is_git_repository_false() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!is_git_repository(temp_dir.path()));
    }
}
