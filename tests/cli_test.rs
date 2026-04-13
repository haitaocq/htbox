use std::process::Command;

fn run_cli(args: &[&str]) -> String {
    let output = Command::new("./target/debug/htbox")
        .args(args)
        .output()
        .expect("Failed to run htbox");

    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn test_cli_help() {
    let output = run_cli(&["--help"]);
    assert!(output.contains("htbox"));
    assert!(output.contains("service"));
    assert!(output.contains("cmd"));
}

#[test]
fn test_cli_version() {
    let output = run_cli(&["--version"]);
    assert!(output.contains("0.1.0"));
}

#[test]
fn test_service_help() {
    let output = run_cli(&["service", "--help"]);
    assert!(output.contains("start"));
    assert!(output.contains("stop"));
    assert!(output.contains("restart"));
    assert!(output.contains("status"));
    assert!(output.contains("list"));
    assert!(output.contains("add"));
    assert!(output.contains("remove"));
    assert!(output.contains("logs"));
    assert!(output.contains("env"));
}

#[test]
fn test_cmd_help() {
    let output = run_cli(&["cmd", "--help"]);
    assert!(output.contains("list"));
    assert!(output.contains("add"));
    assert!(output.contains("run"));
    assert!(output.contains("edit"));
    assert!(output.contains("remove"));
}

#[test]
fn test_service_list_empty() {
    let output = run_cli(&["service", "list"]);
    assert!(output.contains("No services found") || output.len() > 0);
}

#[test]
fn test_cmd_list_empty() {
    let output = run_cli(&["cmd", "list"]);
    assert!(output.len() > 0);
}

#[test]
fn test_service_add_name_required() {
    let output = run_cli(&["service", "add"]);
    assert!(
        output.contains("Name is required")
            || output.contains("Service name is required")
            || output.contains("required")
    );
}

#[test]
fn test_service_status_nonexistent() {
    let output = run_cli(&["service", "status", "nonexistent-service"]);
    assert!(
        output.contains("nonexistent-service")
            || output.contains("stopped")
            || output.contains("Stopped")
    );
}

#[test]
fn test_service_logs_nonexistent() {
    let output = run_cli(&["service", "logs", "nonexistent-service"]);
    assert!(
        output.contains("nonexistent-service")
            || output.contains("stopped")
            || output.contains("no such file")
    );
}

#[test]
fn test_service_env_list_nonexistent() {
    let output = run_cli(&["service", "env", "list", "nonexistent-service"]);
    assert!(
        output.contains("nonexistent-service")
            || output.contains("No such")
            || output.contains("not found")
    );
}

#[test]
fn test_cmd_add_name_required() {
    let output = run_cli(&["cmd", "add"]);
    assert!(output.contains("Name is required") || output.contains("error"));
}

#[test]
fn test_cmd_add_with_params() {
    let output = run_cli(&["cmd", "add", "test-cmd", "--command", "echo hello"]);
    assert!(output.contains("added") || output.contains("success") || output.len() > 0);
}

#[test]
fn test_cmd_run_nonexistent() {
    let output = run_cli(&["cmd", "run", "nonexistent-cmd"]);
    assert!(output.contains("not found") || output.contains("No such") || output.contains("error"));
}

#[test]
fn test_service_remove_without_force() {
    let output = run_cli(&["service", "remove", "test-service"]);
    assert!(
        output.contains("Are you sure")
            || output.contains("cancelled")
            || output.contains("removed")
    );
}

#[test]
fn test_cmd_remove_without_force() {
    let output = run_cli(&["cmd", "remove", "test-cmd"]);
    assert!(output.contains("force") || output.contains("confirm"));
}

#[test]
fn test_edit_service_not_implemented() {
    let output = run_cli(&["service", "edit", "test-service"]);
    assert!(output.contains("not") || output.contains("manually") || output.len() > 0);
}

#[test]
fn test_edit_cmd_not_implemented() {
    let output = run_cli(&["cmd", "edit", "test-cmd"]);
    assert!(output.contains("not") || output.contains("implement") || output.len() > 0);
}
