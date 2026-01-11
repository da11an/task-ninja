use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config_dir = temp_dir.path().join(".taskninja");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("rc");
    fs::write(&config_file, format!("data.location={}\n", db_path.display())).unwrap();
    std::env::set_var("HOME", temp_dir.path().to_str().unwrap());
    temp_dir
}

fn get_task_cmd() -> Command {
    Command::cargo_bin("task").unwrap()
}

#[test]
fn test_task_list_table_formatting() {
    let temp_dir = setup_test_env();
    
    // Create tasks
    get_task_cmd().args(&["add", "Task 1", "project:work", "+urgent"]).assert().success();
    get_task_cmd().args(&["add", "Task 2"]).assert().success();
    
    // List tasks - should show table format
    get_task_cmd().args(&["list"]).assert().success()
        .stdout(predicates::str::contains("ID"))
        .stdout(predicates::str::contains("Description"))
        .stdout(predicates::str::contains("Status"));
    
    drop(temp_dir);
}

#[test]
fn test_task_list_json_format() {
    let temp_dir = setup_test_env();
    
    // Create task
    get_task_cmd().args(&["add", "Task 1"]).assert().success();
    
    // List tasks in JSON format
    get_task_cmd().args(&["list", "--json"]).assert().success()
        .stdout(predicates::str::contains("\"id\""))
        .stdout(predicates::str::contains("\"description\""))
        .stdout(predicates::str::contains("\"status\""));
    
    drop(temp_dir);
}

#[test]
fn test_stack_display_formatting() {
    let temp_dir = setup_test_env();
    
    // Create tasks and add to stack
    get_task_cmd().args(&["add", "Task 1"]).assert().success();
    get_task_cmd().args(&["add", "Task 2"]).assert().success();
    get_task_cmd().args(&["1", "enqueue"]).assert().success();
    get_task_cmd().args(&["2", "enqueue"]).assert().success();
    
    // Show stack - should show formatted display
    get_task_cmd().args(&["stack", "show"]).assert().success()
        .stdout(predicates::str::contains("Stack:"));
    
    drop(temp_dir);
}

#[test]
fn test_stack_json_format() {
    let temp_dir = setup_test_env();
    
    // Create task and add to stack
    get_task_cmd().args(&["add", "Task 1"]).assert().success();
    get_task_cmd().args(&["1", "enqueue"]).assert().success();
    
    // Show stack in JSON format
    get_task_cmd().args(&["stack", "show", "--json"]).assert().success()
        .stdout(predicates::str::contains("\"index\""))
        .stdout(predicates::str::contains("\"task_id\""));
    
    drop(temp_dir);
}

#[test]
fn test_projects_list_table_formatting() {
    let temp_dir = setup_test_env();
    
    // Create projects
    get_task_cmd().args(&["projects", "add", "work"]).assert().success();
    get_task_cmd().args(&["projects", "add", "home"]).assert().success();
    
    // List projects - should show table format
    get_task_cmd().args(&["projects", "list"]).assert().success()
        .stdout(predicates::str::contains("ID"))
        .stdout(predicates::str::contains("Name"));
    
    drop(temp_dir);
}

#[test]
fn test_projects_list_json_format() {
    let temp_dir = setup_test_env();
    
    // Create project
    get_task_cmd().args(&["projects", "add", "work"]).assert().success();
    
    // List projects in JSON format
    get_task_cmd().args(&["projects", "list", "--json"]).assert().success()
        .stdout(predicates::str::contains("\"id\""))
        .stdout(predicates::str::contains("\"name\""));
    
    drop(temp_dir);
}

#[test]
fn test_clock_transition_messages() {
    let temp_dir = setup_test_env();
    
    // Create task and start clock
    get_task_cmd().args(&["add", "Test task"]).assert().success();
    get_task_cmd().args(&["1", "enqueue"]).assert().success();
    
    // Clock in - should show explicit message
    get_task_cmd().args(&["clock", "in"]).assert().success()
        .stdout(predicates::str::contains("Started timing"));
    
    // Clock out - should show explicit message
    get_task_cmd().args(&["clock", "out"]).assert().success()
        .stdout(predicates::str::contains("Stopped timing"));
    
    drop(temp_dir);
}
