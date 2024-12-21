use serde_json::{json, Value};
use std::net::TcpListener;
use std::time::Duration;

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    
    // Start server
    let server = api_server::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    
    format!("http://127.0.0.1:{}", port)
}

async fn test_action(client: &reqwest::Client, app_address: &str, action: &str) -> reqwest::Response {
    client
        .post(&format!("{}/computer", app_address))
        .json(&json!({
            "action": action
        }))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .expect("Failed to execute request")
}