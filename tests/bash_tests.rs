use serde_json::json;
use std::time::Duration;

// Helper function to wait for service to be ready
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

// Helper function to send bash requests
async fn test_bash_request(command: Option<&str>, restart: Option<bool>) -> reqwest::Response {
    let client = reqwest::Client::new();
    
    let payload = json!({
        "command": command,
        "restart": restart
    });
    
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