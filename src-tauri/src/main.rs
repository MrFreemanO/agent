// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod docker;

use docker::DockerManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest;
use serde::{Deserialize, Serialize};
use tauri;

struct DockerState {
    container_id: String,
    app_name: String,
    status: String,
}

impl Default for DockerState {
    fn default() -> Self {
        Self {
            container_id: String::new(),
            app_name: String::from("ConsoleY"),
            status: String::from("stopped"),
        }
    }
}

#[tauri::command]
async fn start_container(
    state: tauri::State<'_, Arc<Mutex<DockerState>>>,
) -> Result<String, String> {
    println!("Starting container...");
    let docker_manager = DockerManager::new().await.map_err(|e| {
        let error_msg = format!("Failed to create DockerManager: {}", e);
        println!("{}", error_msg);
        error_msg
    })?;
    
    // Ensure image exists
    println!("Checking/building Docker image...");
    docker_manager
        .ensure_image("consoleai/desktop:latest")
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to ensure image: {}", e);
            println!("{}", error_msg);
            error_msg
        })?;
    
    // Create and start container
    println!("Creating and starting container...");
    let container_id = docker_manager
        .create_and_start_container()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to create/start container: {}", e);
            println!("{}", error_msg);
            error_msg
        })?;
    
    println!("Container started successfully with ID: {}", container_id);
    let mut state = state.lock().await;
    state.container_id = container_id.clone();
    state.status = "running".to_string();
    
    Ok(container_id)
}

#[tauri::command]
async fn stop_container(
    state: tauri::State<'_, Arc<Mutex<DockerState>>>,
) -> Result<(), String> {
    let docker_manager = DockerManager::new().await.map_err(|e| e.to_string())?;
    let state = state.lock().await;
    
    if !state.container_id.is_empty() {
        docker_manager
            .stop_container(&state.container_id)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn get_container_logs(
    state: tauri::State<'_, Arc<Mutex<DockerState>>>,
) -> Result<String, String> {
    let docker_manager = DockerManager::new().await.map_err(|e| e.to_string())?;
    let state = state.lock().await;
    
    if !state.container_id.is_empty() {
        docker_manager
            .get_container_logs(&state.container_id)
            .await
            .map_err(|e| e.to_string())
    } else {
        Ok("No container running".to_string())
    }
}

#[tauri::command]
async fn restart_container(
    state: tauri::State<'_, Arc<Mutex<DockerState>>>,
) -> Result<(), String> {
    let docker_manager = DockerManager::new().await.map_err(|e| e.to_string())?;
    let state = state.lock().await;
    
    if !state.container_id.is_empty() {
        docker_manager
            .restart_container(&state.container_id)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("No container running".to_string())
    }
}

#[tauri::command]
async fn get_app_info(
    state: tauri::State<'_, Arc<Mutex<DockerState>>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().await;
    Ok(serde_json::json!({
        "name": state.app_name,
        "version": env!("CARGO_PKG_VERSION"),
        "status": state.status,
    }))
}

fn main() {
    println!("Tauri application starting...");

    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Start Tauri application
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(DockerState::default())))
        .invoke_handler(tauri::generate_handler![
            start_container,
            stop_container,
            get_container_logs,
            restart_container,
            get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
