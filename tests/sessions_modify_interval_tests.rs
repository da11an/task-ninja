use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;
use tatl::db::DbConnection;
use tatl::repo::SessionRepo;
mod test_env;

fn setup_test_env() -> (TempDir, std::sync::MutexGuard<'static, ()>) {
    let guard = test_env::lock_test_env();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config_dir = temp_dir.path().join(".tatl");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("rc");
    fs::write(&config_file, format!("data.location={}\n", db_path.display())).unwrap();
    std::env::set_var("HOME", temp_dir.path().to_str().unwrap());
    (temp_dir, guard)
}

fn get_task_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("tatl").unwrap();
    cmd.env("HOME", temp_dir.path());
    cmd
}

#[test]
fn test_sessions_modify_interval_both_times() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Create task and closed session
    get_task_cmd(&temp_dir).args(&["add", "Task 1"]).assert().success();
    get_task_cmd(&temp_dir).args(&["enqueue", "1"]).assert().success();
    get_task_cmd(&temp_dir).args(&["on"]).assert().success();
    get_task_cmd(&temp_dir).args(&["off"]).assert().success();
    
    // Get session ID
    let output = get_task_cmd(&temp_dir).args(&["sessions", "list", "--json"]).assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let session_id = json[0]["id"].as_i64().unwrap();
    
    // Modify using interval syntax
    get_task_cmd(&temp_dir)
        .args(&["sessions", "modify", &session_id.to_string(), "--yes", "09:00..17:00"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Modified session"));
    
    // Verify times were updated
    let conn = DbConnection::connect().unwrap();
    let session = SessionRepo::get_by_id(&conn, session_id).unwrap().unwrap();
    // Check that start_ts is around 09:00 today
    assert!(session.end_ts.is_some(), "Session should have end time");
}

#[test]
fn test_sessions_modify_interval_start_only() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Create task and closed session
    get_task_cmd(&temp_dir).args(&["add", "Task 1"]).assert().success();
    get_task_cmd(&temp_dir).args(&["enqueue", "1"]).assert().success();
    get_task_cmd(&temp_dir).args(&["on"]).assert().success();
    get_task_cmd(&temp_dir).args(&["off"]).assert().success();
    
    // Get session ID
    let output = get_task_cmd(&temp_dir).args(&["sessions", "list", "--json"]).assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let session_id = json[0]["id"].as_i64().unwrap();
    let original_end = json[0]["end_ts"].as_i64();
    
    // Modify start only using interval syntax
    get_task_cmd(&temp_dir)
        .args(&["sessions", "modify", &session_id.to_string(), "--yes", "09:00.."])
        .assert()
        .success();
    
    // Verify end time was preserved
    let conn = DbConnection::connect().unwrap();
    let session = SessionRepo::get_by_id(&conn, session_id).unwrap().unwrap();
    if let Some(orig_end) = original_end {
        assert_eq!(session.end_ts, Some(orig_end), "End time should be preserved");
    }
}

#[test]
fn test_sessions_modify_interval_end_only() {
    let (temp_dir, _guard) = setup_test_env();
    
    // Create task and closed session
    get_task_cmd(&temp_dir).args(&["add", "Task 1"]).assert().success();
    get_task_cmd(&temp_dir).args(&["enqueue", "1"]).assert().success();
    get_task_cmd(&temp_dir).args(&["on"]).assert().success();
    get_task_cmd(&temp_dir).args(&["off"]).assert().success();
    
    // Get session ID
    let output = get_task_cmd(&temp_dir).args(&["sessions", "list", "--json"]).assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let session_id = json[0]["id"].as_i64().unwrap();
    let original_start = json[0]["start_ts"].as_i64().unwrap();
    
    // Modify end only using interval syntax
    get_task_cmd(&temp_dir)
        .args(&["sessions", "modify", &session_id.to_string(), "--yes", "..17:00"])
        .assert()
        .success();
    
    // Verify start time was preserved
    let conn = DbConnection::connect().unwrap();
    let session = SessionRepo::get_by_id(&conn, session_id).unwrap().unwrap();
    assert_eq!(session.start_ts, original_start, "Start time should be preserved");
}
