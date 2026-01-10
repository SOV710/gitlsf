//! # gitlsf - Git Repository Line Counter
//!
//! `gitlsf` is a fast command-line tool for counting lines of code in Git repositories.
//! It uses `git ls-files` to identify tracked files and filters out binary, configuration,
//! and documentation files automatically.
//!
//! ## Features
//!
//! - Fast parallel processing using rayon
//! - Automatic filtering of non-code files
//! - Multiple output modes (verbose, quiet, summary)
//! - Customizable file filtering
//!
//! ## Example Usage
//!
//! ```no_run
//! use gitlsf::{count_repository, count_repository_with_filter, filter::FileFilter};
//!
//! // Count lines in the current directory
//! let summary = count_repository(".").unwrap();
//! println!("Total lines: {}", summary.total_lines);
//! println!("Files counted: {}", summary.file_count);
//!
//! // With custom filter
//! let filter = FileFilter::new().exclude_extension("log");
//! let summary = count_repository_with_filter(".", filter).unwrap();
//! ```
//!
//! ## Modules
//!
//! - [`error`] - Error types for the crate
//! - [`git`] - Git command interaction
//! - [`filter`] - File filtering logic
//! - [`counter`] - Line counting functionality

pub mod counter;
pub mod error;
pub mod filter;
pub mod git;

use std::path::Path;

pub use counter::{CountSummary, FileCount};
pub use error::{GitlsfError, Result};
pub use filter::FileFilter;

/// Counts lines of code in a Git repository.
///
/// This is the main entry point for counting lines in a repository.
/// It uses the default file filter to exclude binary, configuration,
/// and documentation files.
///
/// # Arguments
///
/// * `path` - The path to the Git repository.
///
/// # Returns
///
/// A summary of the counting results, including individual file counts
/// and the total line count.
///
/// # Errors
///
/// Returns an error if:
/// - The path is not within a Git repository
/// - The `git` command fails to execute
///
/// # Example
///
/// ```no_run
/// use gitlsf::count_repository;
///
/// let summary = count_repository(".").unwrap();
/// println!("Total: {} lines in {} files", summary.total_lines, summary.file_count);
/// ```
pub fn count_repository(path: impl AsRef<Path>) -> Result<CountSummary> {
    count_repository_with_filter(path, FileFilter::new())
}

/// Counts lines of code in a Git repository with a custom filter.
///
/// This allows you to customize which files are included in the count.
///
/// # Arguments
///
/// * `path` - The path to the Git repository.
/// * `filter` - The file filter to use.
///
/// # Returns
///
/// A summary of the counting results.
///
/// # Errors
///
/// Returns an error if:
/// - The path is not within a Git repository
/// - The `git` command fails to execute
///
/// # Example
///
/// ```no_run
/// use gitlsf::{count_repository_with_filter, filter::FileFilter};
///
/// let filter = FileFilter::new()
///     .exclude_extension("log")
///     .exclude_filename("generated.rs");
///
/// let summary = count_repository_with_filter(".", filter).unwrap();
/// ```
pub fn count_repository_with_filter(
    path: impl AsRef<Path>,
    filter: FileFilter,
) -> Result<CountSummary> {
    let path = path.as_ref();

    // Get list of tracked files from Git
    let files = git::list_files(path)?;

    // Filter files
    let filtered_files = filter.filter_files(files);

    // Count lines in parallel
    let summary = counter::count_lines_parallel(path, filtered_files);

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_git_repo_with_files() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();

        // Configure git user
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

        // Create various test files
        fs::write(
            path.join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();
        fs::write(path.join("lib.rs"), "pub fn hello() {\n}\n").unwrap();
        fs::write(path.join("README.md"), "# Test\n\nThis is a test.\n").unwrap();
        fs::write(path.join("config.json"), "{\"key\": \"value\"}\n").unwrap();
        fs::write(path.join("image.png"), &[0x89, 0x50, 0x4E, 0x47]).unwrap(); // PNG header

        // Add files to git
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_count_repository() {
        let temp_dir = setup_git_repo_with_files();
        let summary = count_repository(temp_dir.path()).unwrap();

        // Should only count .rs files (main.rs: 3 lines, lib.rs: 2 lines)
        assert_eq!(summary.file_count, 2);
        assert_eq!(summary.total_lines, 5);
    }

    #[test]
    fn test_count_repository_with_custom_filter() {
        let temp_dir = setup_git_repo_with_files();
        let filter = FileFilter::new().exclude_extension("rs");

        let summary = count_repository_with_filter(temp_dir.path(), filter).unwrap();

        // All source files are excluded, only non-filtered files remain
        // But since .json and .md are also filtered by default, should be 0
        assert_eq!(summary.file_count, 0);
    }

    #[test]
    fn test_count_repository_not_a_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = count_repository(temp_dir.path());

        assert!(result.is_err());
        matches!(result.unwrap_err(), GitlsfError::NotAGitRepository);
    }
}
