use serde_json::json;
use std::time::Duration;

// 复用已有的 wait_for_service 函数
async fn wait_for_service() {
    let client = reqwest::Client::new();
    for i in 0..60 {
        if let Ok(response) = client
            .get("http://localhost:8090/health")
            .timeout(Duration::from_secs(2))
            .send()
            .await {
            if response.status().is_success() {
                println!("Service is ready after {} seconds", i);
                return;
            }
        }
        println!("Waiting for service... attempt {}/60", i + 1);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    panic!("Service did not become ready in time");
}

async fn test_bash_request(command: Option<&str>, restart: Option<bool>) -> reqwest::Response {
    let client = reqwest::Client::new();
    
    let payload = json!({
        "command": command,
        "restart": restart
    });
    
    println!("Sending request to /bash with payload: {}", payload);
    
    client
        .post("http://localhost:8090/bash")
        .json(&payload)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to execute request")
}

#[tokio::test]
async fn test_basic_bash_commands() {
    wait_for_service().await;
    
    // Test echo command
    let response = test_bash_request(Some("echo 'Hello World'"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"].as_str().unwrap().trim(), "Hello World");
    
    // Test pwd command
    let response = test_bash_request(Some("pwd"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(!body["data"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_command_with_error() {
    wait_for_service().await;
    
    // Test non-existent command
    let response = test_bash_request(Some("nonexistentcommand"), None).await;
    assert_eq!(response.status().as_u16(), 200); // Still returns 200 as the error is from bash
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["data"].as_str().unwrap().contains("stderr"));
}

#[tokio::test]
async fn test_session_persistence() {
    wait_for_service().await;
    
    // Create a file in the first command
    let response = test_bash_request(Some("echo 'test' > /tmp/test.txt"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    
    // Read the file in the second command
    let response = test_bash_request(Some("cat /tmp/test.txt"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"].as_str().unwrap().trim(), "test");
    
    // Clean up
    let _ = test_bash_request(Some("rm /tmp/test.txt"), None).await;
}

#[tokio::test]
async fn test_session_restart() {
    wait_for_service().await;
    
    // Set a variable
    let response = test_bash_request(Some("TEST_VAR='hello'"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    
    // Echo the variable
    let response = test_bash_request(Some("echo $TEST_VAR"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"].as_str().unwrap().trim(), "hello");
    
    // Restart session
    let response = test_bash_request(None, Some(true)).await;
    assert_eq!(response.status().as_u16(), 200);
    
    // Try to echo the variable again - should be empty after restart
    let response = test_bash_request(Some("echo $TEST_VAR"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"].as_str().unwrap().trim(), "");
}

#[tokio::test]
async fn test_invalid_requests() {
    wait_for_service().await;
    
    // Test missing command
    let response = test_bash_request(None, None).await;
    assert_eq!(response.status().as_u16(), 400);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["data"].as_str().unwrap().contains("No command provided"));
}

#[tokio::test]
async fn test_long_running_command() {
    wait_for_service().await;
    
    // Test a command that takes a few seconds to complete
    let response = test_bash_request(Some("sleep 2; echo 'done'"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["data"].as_str().unwrap().trim(), "done");
}

#[tokio::test]
async fn test_command_output_special_chars() {
    wait_for_service().await;
    
    // Test command with special characters in output
    let response = test_bash_request(Some(r#"echo -e "line1\nline2\tindented""#), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    let output = body["data"].as_str().unwrap();
    assert!(output.contains("line1"));
    assert!(output.contains("line2"));
    assert!(output.contains("\t"));
} 