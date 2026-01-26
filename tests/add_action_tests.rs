use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;
mod test_env;

/// Helper to create a temporary database and set it as the data location
fn setup_test_env() -> (TempDir, std::sync::MutexGuard<'static, ()>) {
    let guard = test_env::lock_test_env();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Create config file
    let config_dir = temp_dir.path().join(".tatl");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("rc");
    fs::write(&config_file, format!("data.location={}\n", db_path.display())).unwrap();
    
    // Set HOME to temp_dir so the config file is found
    std::env::set_var("HOME", temp_dir.path().to_str().unwrap());
    (temp_dir, guard)
}

fn get_task_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("tatl").unwrap();
    cmd.env("HOME", temp_dir.path());
    cmd
}

// =============================================================================
// --finish flag tests
// =============================================================================

#[test]
fn test_add_with_finish_flag() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --finish flag
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--finish", "Already done task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Marked task 1 as completed"));
    
    // Verify task exists and is completed (list --json shows all non-deleted tasks)
    let output = get_task_cmd(&temp_dir)
        .args(&["list", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let tasks = json.as_array().unwrap();
    assert_eq!(tasks.len(), 1, "Should have one task");
    assert_eq!(tasks[0]["status"], "completed", "Task should be completed");
    
    // Verify no pending tasks
    let output = get_task_cmd(&temp_dir)
        .args(&["list", "status:pending", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    // When no tasks match, output is "No tasks found.\n"
    assert!(stdout.contains("No tasks found"), "Should have no pending tasks");
}

#[test]
fn test_add_finish_with_onoff_creates_session_and_completes() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --onoff and --finish
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--onoff", "09:00..10:00", "--finish", "Meeting"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Added session"))
        .stdout(predicate::str::contains("Marked task 1 as completed"));
    
    // Verify session was created
    let output = get_task_cmd(&temp_dir)
        .args(&["sessions", "list", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let sessions = json.as_array().unwrap();
    assert!(!sessions.is_empty(), "Should have at least one session");
    
    // Session should be closed (has end_ts)
    let session = &sessions[0];
    assert!(session["end_ts"] != serde_json::Value::Null, "Session should be closed");
}

#[test]
fn test_add_finish_with_respawn_triggers_respawn() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with respawn rule and --finish
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--finish", "Daily standup", "respawn:daily", "due:09:00"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Marked task 1 as completed"))
        .stdout(predicate::str::contains("Respawned"));
    
    // Verify we have 2 tasks: 1 completed (original) + 1 pending (respawned)
    let output = get_task_cmd(&temp_dir)
        .args(&["list", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let tasks = json.as_array().unwrap();
    assert_eq!(tasks.len(), 2, "Should have two tasks (original + respawned)");
    
    // Check statuses
    let completed_count = tasks.iter().filter(|t| t["status"] == "completed").count();
    let pending_count = tasks.iter().filter(|t| t["status"] == "pending").count();
    assert_eq!(completed_count, 1, "Should have one completed task");
    assert_eq!(pending_count, 1, "Should have one pending task (respawned)");
}

#[test]
fn test_add_finish_conflicts_with_on() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --on and --finish should error
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--on", "--finish", "Conflicting flags"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot use --finish with --on"));
}

#[test]
fn test_add_finish_conflicts_with_enqueue() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --enqueue and --finish should error
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--enqueue", "--finish", "Conflicting flags"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot use --finish with --enqueue"));
}

// =============================================================================
// --close flag tests
// =============================================================================

#[test]
fn test_add_with_close_flag() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --close flag
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--close", "Cancelled request"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Marked task 1 as closed"));
    
    // Verify task exists and is closed (list --json shows all non-deleted tasks)
    let output = get_task_cmd(&temp_dir)
        .args(&["list", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let tasks = json.as_array().unwrap();
    assert_eq!(tasks.len(), 1, "Should have one task");
    assert_eq!(tasks[0]["status"], "closed", "Task should be closed");
    
    // Verify no pending tasks
    let output = get_task_cmd(&temp_dir)
        .args(&["list", "status:pending", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("No tasks found"), "Should have no pending tasks");
}

#[test]
fn test_add_close_with_onoff_creates_session_and_closes() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --onoff and --close (recording effort before closing)
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--onoff", "09:00..10:00", "--close", "Started but abandoned"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Added session"))
        .stdout(predicate::str::contains("Marked task 1 as closed"));
    
    // Verify session was created
    let output = get_task_cmd(&temp_dir)
        .args(&["sessions", "list", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let sessions = json.as_array().unwrap();
    assert!(!sessions.is_empty(), "Should have at least one session");
}

#[test]
fn test_add_close_with_respawn_triggers_respawn() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with respawn rule and --close
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--close", "Daily report", "respawn:daily"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Marked task 1 as closed"))
        .stdout(predicate::str::contains("Respawned"));
    
    // Verify we have 2 tasks: 1 closed (original) + 1 pending (respawned)
    let output = get_task_cmd(&temp_dir)
        .args(&["list", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let tasks = json.as_array().unwrap();
    assert_eq!(tasks.len(), 2, "Should have two tasks (original + respawned)");
    
    // Check statuses
    let closed_count = tasks.iter().filter(|t| t["status"] == "closed").count();
    let pending_count = tasks.iter().filter(|t| t["status"] == "pending").count();
    assert_eq!(closed_count, 1, "Should have one closed task");
    assert_eq!(pending_count, 1, "Should have one pending task (respawned)");
}

#[test]
fn test_add_close_conflicts_with_on() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --on and --close should error
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--on", "--close", "Conflicting flags"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot use --close with --on"));
}

#[test]
fn test_add_close_conflicts_with_enqueue() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with --enqueue and --close should error
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--enqueue", "--close", "Conflicting flags"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot use --close with --enqueue"));
}

// =============================================================================
// Conflict between --finish and --close
// =============================================================================

#[test]
fn test_add_finish_and_close_conflict() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with both --finish and --close should error
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "--finish", "--close", "Double action"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot use both --finish and --close"));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_add_finish_with_project() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Add task with project and --finish
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "-y", "--finish", "Completed work task", "project:work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Marked task"));
    
    // Verify task has project and is completed (list all tasks)
    let output = get_task_cmd(&temp_dir)
        .args(&["list", "--json"])
        .assert()
        .success();
    
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let tasks = json.as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    // Check status and project
    assert_eq!(tasks[0]["status"], "completed");
    assert!(tasks[0]["project_id"] != serde_json::Value::Null, "Project should be assigned");
}

#[test]
fn test_add_finish_flag_after_description() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Flags can appear after description (CLAP trailing_var_arg workaround)
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "Task with flag after", "--finish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Marked task 1 as completed"));
}

#[test]
fn test_add_close_flag_after_description() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Flags can appear after description
    let mut cmd = get_task_cmd(&temp_dir);
    cmd.args(&["add", "Task with flag after", "--close"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task"))
        .stdout(predicate::str::contains("Marked task 1 as closed"));
}
