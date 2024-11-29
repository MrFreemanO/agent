// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod docker;

use docker::DockerManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri;

// 确保 DockerState 是 Send + Sync
#[derive(Debug)]
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
    let docker_manager = Arc::new(DockerManager::new().await.map_err(|e| e.to_string())?);
    
    // 检查是否已有运行的容器
    match docker_manager.list_containers().await {
        Ok(containers) => {
            for container in containers {
                if let Some(names) = container.names {
                    if names.iter().any(|name| name.contains("consoley")) {
                        if let Some(id) = container.id {
                            println!("Found existing container: {}", id);   
                            let mut state_guard = state.lock().await;
                            state_guard.container_id = id.clone();
                            state_guard.status = "running".to_string();
                            return Ok(id);
                        }
                    }
                }
            }
        }
        Err(e) => return Err(e.to_string()),
    }

    // 确保镜像存在
    println!("Checking/building Docker image...");
    let image_tag = if cfg!(debug_assertions) {
        "consoleai/desktop:dev"
    } else {
        "consoleai/desktop:latest"
    };
    
    docker_manager
        .ensure_image(image_tag)
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to ensure image: {}", e);
            println!("{}", error_msg);
            error_msg
        })?;
    
    // 创建并启动容器
    println!("Creating and starting container...");
    let container_id = Arc::clone(&docker_manager)
        .create_and_start_container()
        .await
        .map_err(|e| e.to_string())?;
    
    println!("Container started successfully with ID: {}", container_id);
    
    // 将 MutexGuard 的作用域限制在一个代码块内
    {
        let mut state_guard = state.lock().await;
        state_guard.container_id = container_id.clone();
        state_guard.status = "running".to_string();
    }
    
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
