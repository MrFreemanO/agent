use serde_json::json;
use std::time::Duration;

const API_BASE_URL: &str = "http://localhost:8090";

async fn test_action_with_params(action: &str, text: Option<&str>, coordinate: Option<Vec<i32>>) -> reqwest::Response {
    let client = reqwest::Client::new();
    
    let mut payload = json!({
        "action": action
    });
    
    if let Some(text) = text {
        payload["text"] = json!(text);
    }
    if let Some(coords) = coordinate {
        payload["coordinate"] = json!(coords);
    }
    
    println!("Sending request to {}/computer with payload: {}", API_BASE_URL, payload);
    
    for i in 0..3 {
        match client
            .post(&format!("{}/computer", API_BASE_URL))
            .json(&payload)
            .timeout(Duration::from_secs(10))
            .send()
            .await {
                Ok(response) => {
                    println!("Response status: {}", response.status());
                    if let Ok(text) = response.text().await {
                        println!("Response body: {}", text);
                    }
                    
                    return client
                        .post(&format!("{}/computer", API_BASE_URL))
                        .json(&payload)
                        .timeout(Duration::from_secs(10))
                        .send()
                        .await
                        .expect("Failed to execute request");
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

async fn wait_for_service() {
    let client = reqwest::Client::new();
    for i in 0..60 {
        if let Ok(response) = client
            .get(&format!("{}/health", API_BASE_URL))
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

async fn test_edit_action(
    action: &str,
    path: &str,
    file_text: Option<&str>,
    view_range: Option<Vec<i32>>,
    old_str: Option<&str>,
    new_str: Option<&str>,
    insert_line: Option<i32>,
) -> reqwest::Response {
    let client = reqwest::Client::new();
    
    let mut payload = json!({
        "action": action,
        "path": path
    });
    
    if let Some(text) = file_text {
        payload["file_text"] = json!(text);
    }
    if let Some(range) = view_range {
        payload["view_range"] = json!(range);
    }
    if let Some(old) = old_str {
        payload["old_str"] = json!(old);
    }
    if let Some(new) = new_str {
        payload["new_str"] = json!(new);
    }
    if let Some(line) = insert_line {
        payload["insert_line"] = json!(line);
    }
    
    println!("Sending request to {}/edit with payload: {}", API_BASE_URL, payload);
    
    client
        .post(&format!("{}/edit", API_BASE_URL))
        .json(&payload)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to execute request")
}

#[tokio::test]
async fn test_computer_actions() {
    wait_for_service().await;
    
    let basic_actions = vec![
        ("left_click", true),
        ("right_click", true),
        ("middle_click", true),
        ("double_click", true),
        ("invalid_action", false),
    ];

    for (action, should_succeed) in basic_actions {
        let response = test_action_with_params(action, None, None).await;

        if should_succeed {
            assert_eq!(response.status().as_u16(), 200, "Action '{}' should succeed", action);
            let body: serde_json::Value = response.json().await.expect("Failed to parse response");
            assert!(body.get("data").is_some(), "Response for '{}' should contain data", action);
        } else {
            assert_eq!(response.status().as_u16(), 400, "Action '{}' should fail", action);
        }
    }
}

#[tokio::test]
async fn test_keyboard_actions() {
    let keyboard_tests: Vec<(&str, Option<&str>, Option<Vec<i32>>, bool)> = vec![
        ("key", Some("Return"), None::<Vec<i32>>, true),
        ("type", Some("Hello World"), None::<Vec<i32>>, true),
        ("key", None, None::<Vec<i32>>, false),
        ("type", None, None::<Vec<i32>>, false),
    ];

    for (action, text, _, should_succeed) in keyboard_tests {
        let response = test_action_with_params(action, text, None).await;

        if should_succeed {
            assert_eq!(response.status().as_u16(), 200, 
                "Keyboard action '{}' with text '{:?}' should succeed", action, text);
        } else {
            assert_eq!(response.status().as_u16(), 400, 
                "Keyboard action '{}' with text '{:?}' should fail", action, text);
        }
    }
}

#[tokio::test]
async fn test_mouse_movement_actions() {
    let mouse_tests: Vec<(&str, Option<&str>, Option<Vec<i32>>, bool)> = vec![
        ("mouse_move", None, Some(vec![100, 100]), true),
        ("left_click_drag", None, Some(vec![200, 200]), true),
        // ("mouse_move", None, None, false),
        // ("mouse_move", None, Some(vec![100]), false),
        // ("left_click_drag", None, None, false),
    ];

    for (action, text, coords, should_succeed) in mouse_tests {
        let coords_clone = coords.clone();
        let response = test_action_with_params(action, text, coords).await;

        if should_succeed {
            assert_eq!(response.status().as_u16(), 200, 
                "Mouse action '{}' with coordinates '{:?}' should succeed", action, coords_clone);
        } else {
            assert_eq!(response.status().as_u16(), 400, 
                "Mouse action '{}' with coordinates '{:?}' should fail", action, coords_clone);
        }
    }
}

#[tokio::test]
async fn test_screenshot_response() {
    let response = test_action_with_params("screenshot", None, None).await;
    assert_eq!(response.status().as_u16(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["type"], "base64");
    assert_eq!(body["media_type"], "image/png");
    assert!(!body["data"].as_str().unwrap().is_empty(), "Screenshot data should not be empty");
}

#[tokio::test]
async fn test_cursor_position_response() {
    println!("Starting cursor position test");
    let response = test_action_with_params("cursor_position", None, None).await;
    println!("Testing cursor position...");
    
    let status = response.status().as_u16();
    println!("Response status: {}", status);
    
    let body = response.text().await.expect("Failed to get response text");
    println!("Response body: {}", body);
    
    assert_eq!(status, 200, "Expected 200 OK status");
    
    let body: serde_json::Value = serde_json::from_str(&body).expect("Failed to parse JSON");
    
    assert_eq!(body["type"], "success", "Expected success response type");
    assert_eq!(body["media_type"], "text/plain", "Expected text/plain media type");
    
    let data = body["data"].as_str().expect("Data should be a string");
    assert!(data.contains("X="), "Response should contain X coordinate");
    assert!(data.contains("Y="), "Response should contain Y coordinate");
}

#[tokio::test]
async fn test_file_creation_and_view() {
    wait_for_service().await;
    
    let test_file = "/tmp/test_file.txt";
    let content = "Hello\nWorld\nTest";
    
    // Test file creation
    let response = test_edit_action(
        "create",
        test_file,
        Some(content),
        None,
        None,
        None,
        None
    ).await;
    
    assert_eq!(response.status().as_u16(), 200, "File creation should succeed");
    
    // Test file view
    let response = test_edit_action(
        "view",
        test_file,
        None,
        None,
        None,
        None,
        None
    ).await;
    
    assert_eq!(response.status().as_u16(), 200, "File view should succeed");
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["data"].as_str().unwrap().contains("Hello"), "View should show file content");
    
    // Test view with range
    let response = test_edit_action(
        "view",
        test_file,
        None,
        Some(vec![1, 2]),
        None,
        None,
        None
    ).await;
    
    assert_eq!(response.status().as_u16(), 200, "Range view should succeed");
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["data"].as_str().unwrap().contains("Hello"), "Range view should show specified lines");
}

#[tokio::test]
async fn test_file_modifications() {
    wait_for_service().await;
    
    let test_file = "/tmp/test_edit_file.txt";
    let initial_content = "Line 1\nLine 2\nLine 3";
    
    // Create test file
    let response = test_edit_action(
        "create",
        test_file,
        Some(initial_content),
        None,
        None,
        None,
        None
    ).await;
    assert_eq!(response.status().as_u16(), 200);
    
    // Test str_replace
    let response = test_edit_action(
        "str_replace",
        test_file,
        None,
        None,
        Some("Line 2"),
        Some("Modified Line"),
        None
    ).await;
    
    assert_eq!(response.status().as_u16(), 200, "String replacement should succeed");
    
    // Test insert
    let response = test_edit_action(
        "insert",
        test_file,
        Some("Inserted Line"),
        None,
        None,
        None,
        Some(2)
    ).await;
    
    assert_eq!(response.status().as_u16(), 200, "Line insertion should succeed");
    
    // Test undo_edit
    let response = test_edit_action(
        "undo_edit",
        test_file,
        None,
        None,
        None,
        None,
        None
    ).await;
    
    assert_eq!(response.status().as_u16(), 200, "Undo operation should succeed");
}

#[tokio::test]
async fn test_edit_error_cases() {
    wait_for_service().await;
    
    let test_cases = vec![
        // Invalid action
        ("invalid_action", "/tmp/test.txt", None, None, None, None, None, 400),
        // Missing file_text for create
        ("create", "/tmp/test.txt", None, None, None, None, None, 400),
        // Invalid view range
        ("view", "/tmp/test.txt", None, Some(vec![1]), None, None, None, 400),
        // Missing old_str for str_replace
        ("str_replace", "/tmp/test.txt", None, None, None, Some("new"), None, 400),
        // Invalid insert_line
        ("insert", "/tmp/test.txt", None, None, None, Some("text"), Some(-1), 400),
    ];
    
    for (action, path, file_text, view_range, old_str, new_str, insert_line, expected_status) in test_cases {
        let response = test_edit_action(
            action,
            path,
            file_text,
            view_range,
            old_str,
            new_str,
            insert_line
        ).await;
        
        assert_eq!(
            response.status().as_u16(),
            expected_status,
            "Test case for {} should return status code {}",
            action,
            expected_status
        );
    }
} 