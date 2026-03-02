//! Integration tests for the CLI binary.
//!
//! Verifies that the `aplan` binary exists and responds to basic flags.
//!
//! This test is registered as a [[test]] in the agentic-planning-cli crate
//! so that CARGO_BIN_EXE_aplan is available.

use std::process::Command;
use tempfile::tempdir;

/// Get a Command pointing to the `aplan` binary.
fn aplan_binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_aplan"))
}

#[test]
fn cli_responds_to_help() {
    let output = aplan_binary()
        .arg("--help")
        .output()
        .expect("failed to execute aplan --help");

    assert!(
        output.status.success(),
        "aplan --help should exit with success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aplan") || stdout.contains("AgenticPlanning") || stdout.contains("Usage"),
        "aplan --help output should contain usage information, got: {stdout}"
    );
}

#[test]
fn cli_responds_to_version() {
    let output = aplan_binary()
        .arg("--version")
        .output()
        .expect("failed to execute aplan --version");

    assert!(
        output.status.success(),
        "aplan --version should exit with success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("0.") || stdout.contains("agentic-planning"),
        "aplan --version output should contain a version number, got: {stdout}"
    );
}

#[test]
fn cli_goal_get_invalid_id_exits_nonzero() {
    let dir = tempdir().expect("failed to create temp dir");
    let aplan_path = dir.path().join("test.aplan");
    let aplan_path_str = aplan_path.to_string_lossy().to_string();

    // Create a goal first to ensure the file exists
    let create = aplan_binary()
        .args([
            "--file",
            &aplan_path_str,
            "goal",
            "create",
            "--title",
            "Test goal",
            "--intention",
            "Testing",
        ])
        .output()
        .expect("failed to execute aplan goal create");

    assert!(
        create.status.success(),
        "aplan goal create should succeed, stderr: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    // Try to get a goal with an invalid UUID
    let invalid = aplan_binary()
        .args([
            "--file",
            &aplan_path_str,
            "goal",
            "get",
            "--id",
            "not-a-valid-uuid",
        ])
        .output()
        .expect("failed to execute aplan goal get with invalid id");

    assert!(
        !invalid.status.success(),
        "aplan goal get with invalid id should exit non-zero"
    );
}
