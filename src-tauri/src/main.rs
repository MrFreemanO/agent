// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod docker;
mod resources;

use docker::DockerManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri;
use tauri::Manager;

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
    
    if let Err(e) = docker_manager.ensure_image(image_tag).await {
        return Err(e.to_string());
    }
    
    // 创建并启动容器
    println!("Creating and starting container...");
    let container_id = match docker_manager.create_and_start_container().await {
        Ok(id) => id,
        Err(e) => return Err(e.to_string()),
    };
    
    println!("Container started successfully with ID: {}", container_id);
    
    // 更新状态
    {
        let mut state_guard = state.lock().await;
        state_guard.container_id = container_id.clone();
        state_guard.status = "running".to_string();
    }
    
    // 添加API服务检查
    let check_api = async {
        let client = reqwest::Client::new();
        for _ in 0..30 {  // 尝试30次，每次等待1秒
            match client.get("http://localhost:8090/health").send().await {
                Ok(response) if response.status().is_success() => {
                    println!("API server is ready");
                    return Ok(());
                }
                _ => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
        Err("API server failed to start".to_string())
    };

    // 等待API服务就绪
    if let Err(e) = check_api.await {
        return Err(e);
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
    tauri::Builder::default()
        .setup(|app| {
            // 使用新的 API 获取资源路径
            let resource_path = app.handle().path().resource_dir()
                .expect("Failed to get resource dir");
            
            println!("Resource path: {:?}", resource_path);
            
            // 启动容器管理
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_docker().await {
                    eprintln!("Failed to setup Docker: {}", e);
                }
            });
            
            Ok(())
        })
        .manage(Arc::new(Mutex::new(DockerState::default())))  // 添加状态管理
        .invoke_handler(tauri::generate_handler![
            start_container,
            stop_container,
            get_container_logs,
            restart_container,
            get_app_info
        ])  // 注册所有命令
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn setup_docker() -> Result<(), String> {
    println!("Initializing Docker setup...");
    
    let docker_manager = Arc::new(DockerManager::new().await.map_err(|e| e.to_string())?);
    
    // 确保镜像存在
    let image_tag = if cfg!(debug_assertions) {
        "consoleai/desktop:dev"
    } else {
        "consoleai/desktop:latest"
    };
    
    println!("Ensuring Docker image exists: {}", image_tag);
    if let Err(e) = docker_manager.ensure_image(image_tag).await {
        eprintln!("Failed to ensure Docker image: {:?}", e);
        return Err(format!("{:?}", e));
    }
    
    // 创建并启动容器
    println!("Creating and starting container...");
    match docker_manager.create_and_start_container().await {
        Ok(container_id) => {
            println!("Container started successfully with ID: {}", container_id);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to start container: {:?}", e);
            Err(format!("{:?}", e))
        }
    }
}
