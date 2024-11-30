// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod docker;
mod resources;

use docker::DockerManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri;
use tauri::{Manager, RunEvent, WindowEvent, Emitter};

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
    app: tauri::AppHandle,
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
        println!("Debug mode detected, using dev image");
        "consoleai/desktop:dev"
    } else {
        println!("Release mode detected, using latest image");
        "consoleai/desktop:latest"
    };
    
    if let Err(e) = docker_manager.ensure_image(&app, image_tag).await {
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
    
    // 发送服务就绪事件
    app.emit("vnc-ready", ()).map_err(|e| e.to_string())?;
    
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
    let mut app = tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            
            // 启动容器管理
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_docker(&app_handle_clone).await {
                    eprintln!("Failed to setup Docker: {}", e);
                }
            });
            
            Ok(())
        })
        .manage(Arc::new(Mutex::new(DockerState::default())))
        .invoke_handler(tauri::generate_handler![
            start_container,
            stop_container,
            get_container_logs,
            restart_container,
            get_app_info
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // 添加事件处理
    app.run(|app_handle, event| {
        if let RunEvent::WindowEvent { label: _, event: WindowEvent::CloseRequested { .. }, .. } = event {
            // 执行清理
            tauri::async_runtime::block_on(async {
                if let Err(e) = cleanup_docker().await {
                    eprintln!("Failed to cleanup Docker: {}", e);
                }
            });
        }
    });
}

async fn setup_docker(app: &tauri::AppHandle) -> Result<(), String> {
    println!("Initializing Docker setup...");
    
    let docker_manager = Arc::new(DockerManager::new().await.map_err(|e| {
        let err_msg = format!("Failed to create Docker manager: {}", e);
        println!("{}", err_msg);
        err_msg
    })?);
    
    // 确保镜像存在
    let image_tag = if cfg!(debug_assertions) {
        println!("Debug mode detected, using dev image");
        "consoleai/desktop:dev"
    } else {
        println!("Release mode detected, using latest image");
        "consoleai/desktop:latest"
    };
    
    println!("Ensuring Docker image exists: {}", image_tag);
    if let Err(e) = docker_manager.ensure_image(app, image_tag).await {
        let err_msg = format!("Failed to ensure Docker image: {:?}", e);
        println!("{}", err_msg);
        return Err(err_msg);
    }
    
    // 创建并启动容器
    println!("Creating and starting container...");
    match docker_manager.create_and_start_container().await {
        Ok(container_id) => {
            println!("Container started successfully with ID: {}", container_id);
            
            // 发送服务就绪事件
            app.emit("vnc-ready", ()).map_err(|e| e.to_string())?;
            
            Ok(())
        }
        Err(e) => {
            let err_msg = format!("Failed to start container: {:?}", e);
            println!("{}", err_msg);
            Err(err_msg)
        }
    }
}

async fn cleanup_docker() -> Result<(), String> {
    let docker_manager = Arc::new(DockerManager::new().await.map_err(|e| e.to_string())?);
    docker_manager.cleanup().await.map_err(|e| e.to_string())
}
