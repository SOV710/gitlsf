//! gitlsf - A fast Git repository line counter.
//!
//! This is the CLI entry point for the gitlsf tool.

use std::process::ExitCode;

use clap::Parser;

use gitlsf::{CountSummary, count_repository};

/// A fast Git repository line counter.
///
/// Counts lines of code in Git repositories, automatically filtering out
/// binary files, configuration files, and documentation.
#[derive(Parser, Debug)]
#[command(name = "gitlsf")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Git repository (defaults to current directory)
    #[arg(default_value = ".")]
    path: String,

    /// Verbose mode - show each file with its line count (default)
    #[arg(short, long, conflicts_with_all = ["quiet", "summary"])]
    verbose: bool,

    /// Quiet mode - only show the total line count
    #[arg(short, long, conflicts_with_all = ["verbose", "summary"])]
    quiet: bool,

    /// Summary mode - show total lines and file count
    #[arg(short, long, conflicts_with_all = ["verbose", "quiet"])]
    summary: bool,
}

/// Output mode for the line count results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    /// Show each file with its line count.
    Verbose,
    /// Only show the total line count.
    Quiet,
    /// Show total lines and file count.
    Summary,
}

impl Args {
    /// Determines the output mode based on the command-line arguments.
    fn output_mode(&self) -> OutputMode {
        if self.quiet {
            OutputMode::Quiet
        } else if self.summary {
            OutputMode::Summary
        } else {
            // Default to verbose (including when -v is explicitly passed)
            OutputMode::Verbose
        }
    }
}

/// Prints the results according to the specified output mode.
fn print_results(summary: &CountSummary, mode: OutputMode) {
    match mode {
        OutputMode::Verbose => {
            // Sort files by path for consistent output
            let mut files = summary.files.clone();
            files.sort_by(|a, b| a.path.cmp(&b.path));

            // Calculate the width needed for line numbers
            let max_lines = files.iter().map(|f| f.lines).max().unwrap_or(0);
            let max_lines = max_lines.max(summary.total_lines);
            let width = max_lines.to_string().len().max(4);

            for file in &files {
                println!("{:>width$} {}", file.lines, file.path);
            }
            println!("{:>width$} total", summary.total_lines);
        }
        OutputMode::Quiet => {
            println!("{}", summary.total_lines);
        }
        OutputMode::Summary => {
            println!("Files: {}", summary.file_count);
            println!("Lines: {}", summary.total_lines);
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    let mode = args.output_mode();

    match count_repository(&args.path) {
        Ok(summary) => {
            print_results(&summary, mode);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_mode_default_is_verbose() {
        let args = Args {
            path: ".".to_string(),
            verbose: false,
            quiet: false,
            summary: false,
        };
        assert_eq!(args.output_mode(), OutputMode::Verbose);
    }

    #[test]
    fn test_output_mode_verbose() {
        let args = Args {
            path: ".".to_string(),
            verbose: true,
            quiet: false,
            summary: false,
        };
        assert_eq!(args.output_mode(), OutputMode::Verbose);
    }

    #[test]
    fn test_output_mode_quiet() {
        let args = Args {
            path: ".".to_string(),
            verbose: false,
            quiet: true,
            summary: false,
        };
        assert_eq!(args.output_mode(), OutputMode::Quiet);
    }

    #[test]
    fn test_output_mode_summary() {
        let args = Args {
            path: ".".to_string(),
            verbose: false,
            quiet: false,
            summary: true,
        };
        assert_eq!(args.output_mode(), OutputMode::Summary);
    }
}
