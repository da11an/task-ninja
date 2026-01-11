// Acceptance tests for Task Ninja
// These implement the Given/When/Then scenarios from Section 11 of the design document

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::env;
use std::fs;

/// Helper to create a temporary database and set it as the data location
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Create config file
    let config_dir = temp_dir.path().join(".taskninja");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("rc");
    fs::write(&config_file, format!("data.location={}\n", db_path.display())).unwrap();
    
    temp_dir
}

/// Helper to create a new command with test environment
fn new_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("task").unwrap();
    cmd.env("HOME", temp_dir.path());
    cmd
}

// Section 11.8: Projects

#[test]
fn acceptance_project_rename_errors_if_target_exists() {
    // Given project `work` exists
    // And project `office` exists
    // When `task projects rename work office`
    // Then exit code is 1
    // And message indicates project already exists
    
    let temp_dir = setup_test_env();
    
    new_cmd(&temp_dir)
        .args(&["projects", "add", "work"])
        .assert()
        .success();
    
    new_cmd(&temp_dir)
        .args(&["projects", "add", "office"])
        .assert()
        .success();
    
    new_cmd(&temp_dir)
        .args(&["projects", "rename", "work", "office"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn acceptance_project_rename_with_force_merges_projects() {
    // Given project `temp` exists with tasks 10, 11
    // And project `work` exists with task 12
    // When `task projects rename temp work --force`
    // Then project `temp` no longer exists
    // And tasks 10, 11, 12 all reference project `work`
    // And project `work` still exists
    
    let temp_dir = setup_test_env();
    
    // Create projects
    new_cmd(&temp_dir)
        .args(&["projects", "add", "temp"])
        .assert()
        .success();
    
    new_cmd(&temp_dir)
        .args(&["projects", "add", "work"])
        .assert()
        .success();
    
    // TODO: Create tasks once task add is implemented
    // For now, just verify the merge works
    new_cmd(&temp_dir)
        .args(&["projects", "rename", "temp", "work", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Merged"));
    
    // Verify temp no longer exists
    new_cmd(&temp_dir)
        .args(&["projects", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("temp").not());
    
    // Verify work still exists
    new_cmd(&temp_dir)
        .args(&["projects", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("work"));
}

// More acceptance tests will be added as features are implemented
