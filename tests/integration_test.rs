//! Integration tests for the gitlsf CLI tool.

#![allow(deprecated)]

use std::fs;
use std::process::Command;

use assert_cmd::Command as AssertCmd;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper function to set up a Git repository with test files.
fn setup_git_repo() -> TempDir {
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

    // Create source files
    fs::create_dir_all(path.join("src")).unwrap();
    fs::write(
        path.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )
    .unwrap();
    fs::write(
        path.join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();
    fs::write(
        path.join("src/utils.rs"),
        "pub fn helper() {\n    // do something\n}\n",
    )
    .unwrap();

    // Create non-code files that should be filtered
    fs::write(path.join("README.md"), "# Test Project\n\nA test.\n").unwrap();
    fs::write(path.join("config.json"), "{\"key\": \"value\"}\n").unwrap();
    fs::write(path.join("LICENSE"), "MIT License\n").unwrap();

    // Add files to git
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();

    temp_dir
}

#[test]
fn test_help() {
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Git repository line counter"));
}

#[test]
fn test_version() {
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_verbose_mode_default() {
    let temp_dir = setup_git_repo();
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();

    cmd.arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains("src/utils.rs"))
        .stdout(predicate::str::contains("total"));
}

#[test]
fn test_verbose_mode_explicit() {
    let temp_dir = setup_git_repo();
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();

    cmd.arg("-v")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("total"));
}

#[test]
fn test_quiet_mode() {
    let temp_dir = setup_git_repo();
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();

    cmd.arg("-q")
        .arg(temp_dir.path())
        .assert()
        .success()
        // Should only contain the total number
        .stdout(predicate::str::is_match(r"^\d+\n$").unwrap());
}

#[test]
fn test_summary_mode() {
    let temp_dir = setup_git_repo();
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();

    cmd.arg("-s")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Files:"))
        .stdout(predicate::str::contains("Lines:"));
}

#[test]
fn test_not_a_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();

    cmd.arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a Git repository"));
}

#[test]
fn test_filters_binary_files() {
    let temp_dir = setup_git_repo();
    let path = temp_dir.path();

    // Add a binary file
    fs::write(path.join("image.png"), &[0x89, 0x50, 0x4E, 0x47]).unwrap();
    Command::new("git")
        .args(["add", "image.png"])
        .current_dir(path)
        .output()
        .unwrap();

    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();
    cmd.arg(temp_dir.path())
        .assert()
        .success()
        // Should not contain the binary file
        .stdout(predicate::str::contains("image.png").not());
}

#[test]
fn test_filters_config_files() {
    let temp_dir = setup_git_repo();
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();

    cmd.arg(temp_dir.path())
        .assert()
        .success()
        // Should not contain config.json or README.md
        .stdout(predicate::str::contains("config.json").not())
        .stdout(predicate::str::contains("README.md").not())
        .stdout(predicate::str::contains("LICENSE").not());
}

#[test]
fn test_line_count_accuracy() {
    let temp_dir = setup_git_repo();

    // main.rs: 3 lines, lib.rs: 3 lines, utils.rs: 3 lines = 9 total
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();
    cmd.arg("-q")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout("9\n");
}

#[test]
fn test_summary_file_count() {
    let temp_dir = setup_git_repo();

    // 3 .rs files should be counted
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();
    cmd.arg("-s")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Files: 3"));
}

#[test]
fn test_conflicting_flags() {
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();
    cmd.args(["-v", "-q"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_empty_repo() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Initialize git repo without any files
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();

    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();
    cmd.arg("-q")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout("0\n");
}

#[test]
fn test_current_directory_default() {
    // This test verifies that the tool works with "." as default
    // We can't easily test this in an isolated way, so we just verify
    // the command runs without errors in a git repo
    let mut cmd = AssertCmd::cargo_bin("gitlsf").unwrap();

    // Run from the project root (which is a git repo)
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("-s")
        .assert()
        .success()
        .stdout(predicate::str::contains("Files:"))
        .stdout(predicate::str::contains("Lines:"));
}
