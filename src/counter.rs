//! Line counting module.
//!
//! This module provides functionality for counting lines in files,
//! with support for parallel processing to handle large repositories efficiently.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use memchr::memchr_iter;
use rayon::prelude::*;

use crate::error::{GitlsError, Result};

/// Buffer size for reading files (64KB).
const BUFFER_SIZE: usize = 64 * 1024;

/// Result of counting lines in a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCount {
    /// The path to the file.
    pub path: String,
    /// The number of lines in the file.
    pub lines: usize,
}

impl FileCount {
    /// Creates a new file count result.
    pub fn new(path: impl Into<String>, lines: usize) -> Self {
        Self {
            path: path.into(),
            lines,
        }
    }
}

/// Summary of counting results for multiple files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CountSummary {
    /// Individual file counts.
    pub files: Vec<FileCount>,
    /// Total number of lines across all files.
    pub total_lines: usize,
    /// Total number of files counted.
    pub file_count: usize,
}

impl CountSummary {
    /// Creates a new count summary from file counts.
    pub fn from_counts(files: Vec<FileCount>) -> Self {
        let total_lines = files.iter().map(|f| f.lines).sum();
        let file_count = files.len();

        Self {
            files,
            total_lines,
            file_count,
        }
    }
}

/// Counts lines in a single file using fast byte-level scanning.
///
/// # Arguments
///
/// * `base_path` - The base directory of the repository.
/// * `file_path` - The path to the file relative to the base directory.
///
/// # Returns
///
/// The number of lines in the file.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
///
/// # Example
///
/// ```no_run
/// use gitls::counter::count_lines;
///
/// let lines = count_lines(".", "src/main.rs").unwrap();
/// println!("Lines: {}", lines);
/// ```
pub fn count_lines(base_path: impl AsRef<Path>, file_path: impl AsRef<Path>) -> Result<usize> {
    let base = base_path.as_ref();
    let file = file_path.as_ref();
    let full_path = base.join(file);

    let mut f = File::open(&full_path).map_err(|e| GitlsError::io(&full_path, e))?;

    let mut buffer = [0u8; BUFFER_SIZE];
    let mut count = 0usize;
    let mut last_byte = None;

    loop {
        let bytes_read = f
            .read(&mut buffer)
            .map_err(|e| GitlsError::io(&full_path, e))?;

        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        count += memchr_iter(b'\n', chunk).count();
        last_byte = chunk.last().copied();
    }

    // If file is non-empty and doesn't end with newline, count the last line
    if let Some(b) = last_byte
        && b != b'\n'
    {
        count += 1;
    }

    Ok(count)
}

/// Counts lines in multiple files in parallel.
///
/// Files that cannot be read are skipped.
///
/// # Arguments
///
/// * `base_path` - The base directory of the repository.
/// * `files` - An iterator of file paths relative to the base directory.
///
/// # Returns
///
/// A summary of the counting results.
///
/// # Example
///
/// ```no_run
/// use gitls::counter::count_lines_parallel;
///
/// let files = vec!["src/main.rs", "src/lib.rs"];
/// let summary = count_lines_parallel(".", files);
/// println!("Total lines: {}", summary.total_lines);
/// ```
pub fn count_lines_parallel<I, S>(base_path: impl AsRef<Path>, files: I) -> CountSummary
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let base = base_path.as_ref();
    let files: Vec<String> = files.into_iter().map(|f| f.as_ref().to_string()).collect();

    let counts: Vec<FileCount> = files
        .par_iter()
        .filter_map(|file_path| {
            match count_lines(base, file_path) {
                Ok(lines) => Some(FileCount::new(file_path.clone(), lines)),
                Err(_) => None, // Skip files that can't be read
            }
        })
        .collect();

    CountSummary::from_counts(counts)
}

/// Counts lines in multiple files sequentially.
///
/// This is useful for testing or when parallel processing is not desired.
///
/// # Arguments
///
/// * `base_path` - The base directory of the repository.
/// * `files` - An iterator of file paths relative to the base directory.
///
/// # Returns
///
/// A summary of the counting results.
pub fn count_lines_sequential<I, S>(base_path: impl AsRef<Path>, files: I) -> CountSummary
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let base = base_path.as_ref();
    let files: Vec<String> = files.into_iter().map(|f| f.as_ref().to_string()).collect();

    let counts: Vec<FileCount> = files
        .iter()
        .filter_map(|file_path| match count_lines(base, file_path) {
            Ok(lines) => Some(FileCount::new(file_path.clone(), lines)),
            Err(_) => None,
        })
        .collect();

    CountSummary::from_counts(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_files() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create test files with known line counts
        fs::write(path.join("one_line.txt"), "single line\n").unwrap();
        fs::write(path.join("three_lines.txt"), "line1\nline2\nline3\n").unwrap();
        fs::write(path.join("empty.txt"), "").unwrap();
        fs::write(path.join("no_newline.txt"), "no newline at end").unwrap();

        // Create a subdirectory with files
        fs::create_dir(path.join("src")).unwrap();
        fs::write(
            path.join("src/main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_count_lines_single_line() {
        let temp_dir = setup_test_files();
        let count = count_lines(temp_dir.path(), "one_line.txt").unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_lines_multiple_lines() {
        let temp_dir = setup_test_files();
        let count = count_lines(temp_dir.path(), "three_lines.txt").unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_lines_empty_file() {
        let temp_dir = setup_test_files();
        let count = count_lines(temp_dir.path(), "empty.txt").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_lines_no_trailing_newline() {
        let temp_dir = setup_test_files();
        let count = count_lines(temp_dir.path(), "no_newline.txt").unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_lines_subdirectory() {
        let temp_dir = setup_test_files();
        let count = count_lines(temp_dir.path(), "src/main.rs").unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_lines_nonexistent_file() {
        let temp_dir = setup_test_files();
        let result = count_lines(temp_dir.path(), "nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_count_lines_parallel() {
        let temp_dir = setup_test_files();
        let files = vec![
            "one_line.txt",
            "three_lines.txt",
            "src/main.rs",
            "nonexistent.txt", // Should be skipped
        ];

        let summary = count_lines_parallel(temp_dir.path(), files);

        assert_eq!(summary.file_count, 3);
        assert_eq!(summary.total_lines, 1 + 3 + 3);
    }

    #[test]
    fn test_count_lines_sequential() {
        let temp_dir = setup_test_files();
        let files = vec!["one_line.txt", "three_lines.txt"];

        let summary = count_lines_sequential(temp_dir.path(), files);

        assert_eq!(summary.file_count, 2);
        assert_eq!(summary.total_lines, 4);
    }

    #[test]
    fn test_file_count_new() {
        let fc = FileCount::new("test.rs", 100);
        assert_eq!(fc.path, "test.rs");
        assert_eq!(fc.lines, 100);
    }

    #[test]
    fn test_count_summary_from_counts() {
        let counts = vec![
            FileCount::new("a.rs", 10),
            FileCount::new("b.rs", 20),
            FileCount::new("c.rs", 30),
        ];

        let summary = CountSummary::from_counts(counts);

        assert_eq!(summary.file_count, 3);
        assert_eq!(summary.total_lines, 60);
    }
}
