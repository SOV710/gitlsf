//! Error types for the gitls crate.
//!
//! This module defines all error types used throughout the application,
//! providing clear and informative error messages for various failure scenarios.

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for gitls operations.
#[derive(Error, Debug)]
pub enum GitlsError {
    /// Error executing a Git command.
    #[error("Git command failed: {message}")]
    Git {
        /// Description of what went wrong.
        message: String,
        /// The underlying IO error, if any.
        #[source]
        source: Option<std::io::Error>,
    },

    /// Error reading a file.
    #[error("Failed to read file '{path}': {source}")]
    Io {
        /// The path to the file that couldn't be read.
        path: PathBuf,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Error decoding UTF-8 content.
    #[error("Invalid UTF-8 in '{context}': {source}")]
    Utf8 {
        /// Context describing where the UTF-8 error occurred.
        context: String,
        /// The underlying UTF-8 error.
        #[source]
        source: std::string::FromUtf8Error,
    },

    /// Not a Git repository.
    #[error("Not a Git repository (or any parent up to mount point)")]
    NotAGitRepository,
}

/// A specialized Result type for gitls operations.
pub type Result<T> = std::result::Result<T, GitlsError>;

impl GitlsError {
    /// Creates a new Git error with a message.
    pub fn git(message: impl Into<String>) -> Self {
        Self::Git {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a new Git error with a message and source error.
    pub fn git_with_source(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::Git {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Creates a new IO error.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Creates a new UTF-8 error.
    pub fn utf8(context: impl Into<String>, source: std::string::FromUtf8Error) -> Self {
        Self::Utf8 {
            context: context.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_error_display() {
        let err = GitlsError::git("test error");
        assert_eq!(err.to_string(), "Git command failed: test error");
    }

    #[test]
    fn test_not_a_git_repository_display() {
        let err = GitlsError::NotAGitRepository;
        assert_eq!(
            err.to_string(),
            "Not a Git repository (or any parent up to mount point)"
        );
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = GitlsError::io("/some/path", io_err);
        assert!(err.to_string().contains("/some/path"));
    }

    #[test]
    fn test_utf8_error_display() {
        let invalid_utf8 = vec![0xff, 0xfe];
        let utf8_err = String::from_utf8(invalid_utf8).unwrap_err();
        let err = GitlsError::utf8("git output", utf8_err);
        assert!(err.to_string().contains("git output"));
    }
}
