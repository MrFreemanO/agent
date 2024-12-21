use serde_json::json;
use std::time::Duration;

// Helper function to wait for service to be ready
async fn wait_for_service() {
    let client = reqwest::Client::new();
    for i in 0..60 {
        if let Ok(response) = client
            .get("http://localhost:8090/health")
            .timeout(Duration::from_secs(5))
            .send()
            .await {
            if response.status().is_success() {
                tokio::time::sleep(Duration::from_secs(2)).await;
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
    
    for i in 0..3 {
        match client
            .post("http://localhost:8090/bash")
            .json(&payload)
            .timeout(Duration::from_secs(30))
            .send()
            .await {
                Ok(response) => {
                    return response;
                },
                Err(e) => {
                    println!("Request failed (attempt {}/3): {}", i + 1, e);
                    if i == 2 {
                        panic!("All retry attempts failed: {}", e);
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
    }
    
    panic!("Should not reach here");
}

#[tokio::test]
async fn test_basic_bash_commands() {
    wait_for_service().await;
    
    // Test echo command
    let response = test_bash_request(Some("echo 'Hello World'"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.expect("Failed to get response text");
    assert_eq!(body.trim(), "Hello World");
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
    let body = response.text().await.expect("Failed to get response text");
    assert_eq!(body.trim(), "hello");
    
    // Restart session
    let response = test_bash_request(None, Some(true)).await;
    assert_eq!(response.status().as_u16(), 200);
    
    // Try to echo the variable again - should be empty after restart
    let response = test_bash_request(Some("echo $TEST_VAR"), None).await;
    assert_eq!(response.status().as_u16(), 200);
    let body = response.text().await.expect("Failed to get response text");
    assert_eq!(body.trim(), "");
} 