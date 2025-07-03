use std::process::Command;
use std::fs;

fn create_test_file(name: &str, content: &str) -> String {
    let test_file = format!("/tmp/{}", name);
    fs::write(&test_file, content).unwrap();
    test_file
}

#[test]
fn test_check_valid_typescript() {
    let test_file = create_test_file("valid.ts", "const x: number = 42;");
    
    let output = Command::new("cargo")
        .args(["run", "--bin", "seal", "check", &test_file])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Type checking passed"));
    
    fs::remove_file(test_file).ok();
}

#[test]
fn test_check_invalid_typescript() {
    let test_file = create_test_file("invalid.ts", "return 42;"); // return outside function
    
    let output = Command::new("cargo")
        .args(["run", "--bin", "seal", "check", &test_file])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Type checking failed"));
    
    fs::remove_file(test_file).ok();
}

#[test]
fn test_check_nonexistent_file() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "seal", "check", "/nonexistent/file.ts"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error:"));
}

#[test]
fn test_check_parse_error() {
    let test_file = create_test_file("parse_error.ts", "const x: = 42;"); // syntax error
    
    let output = Command::new("cargo")
        .args(["run", "--bin", "seal", "check", &test_file])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Parse error"));
    
    fs::remove_file(test_file).ok();
}