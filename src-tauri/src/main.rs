// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod docker;
mod resources;

use docker::DockerManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri;
use tauri::{Manager, RunEvent, WindowEvent, Emitter};
use std::io::Write;
use chrono;

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

// 添加日志宏定义
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{}", msg);
        crate::log_to_file(&msg);
    }};
}

// 修改日志函数
fn log_to_file(msg: &str) {
    let log_path = std::env::temp_dir().join("consoley").join("app.log");
    
    // 尝试创建日志目录，忽略可能的错误
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    // 追加日志内容
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path) 
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

#[tauri::command]
async fn start_container(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Mutex<DockerState>>>,
) -> Result<String, String> {
    log!("Starting container process...");
    let docker_manager = Arc::new(DockerManager::new().await.map_err(|e| e.to_string())?);
    
    // 检查是否已有运行的容器
    log!("Checking for existing containers...");
    match docker_manager.list_containers().await {
        Ok(containers) => {
            for container in containers {
                if let Some(names) = container.names {
                    if names.iter().any(|name| name.contains("consoley")) {
                        if let Some(id) = container.id {
                            log!("Found existing container: {}", id);   
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
    log!("Checking/building Docker image...");
    let image_tag = if cfg!(debug_assertions) {
        log!("Debug mode detected, using dev image");
        "consoleai/desktop:dev"
    } else {
        log!("Release mode detected, using latest image");
        "consoleai/desktop:latest"
    };
    
    if let Err(e) = docker_manager.ensure_image(&app, image_tag).await {
        let err_msg = format!("Failed to ensure image: {}", e);
        log!("{}", err_msg);
        return Err(err_msg);
    }
    
    // 创建并启动容器
    log!("Creating and starting container...");
    let container_id = match docker_manager.create_and_start_container().await {
        Ok(id) => {
            log!("Container created with ID: {}", id);
            id
        },
        Err(e) => return Err(format!("Failed to create container: {}", e)),
    };
    
    // 更新状态
    {
        let mut state_guard = state.lock().await;
        state_guard.container_id = container_id.clone();
        state_guard.status = "running".to_string();
    }
    
    // 添加API服务检查
    log!("Checking API service...");
    let check_api = async {
        let client = reqwest::Client::new();
        for i in 0..30 {
            log!("API check attempt {}/30", i + 1);
            match client.get("http://localhost:8090/health").send().await {
                Ok(response) if response.status().is_success() => {
                    log!("API server is ready");
                    return Ok(());
                }
                _ => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
        Err("API server failed to start".to_string())
    };

    if let Err(e) = check_api.await {
        return Err(format!("API service check failed: {}", e));
    }
    
    log!("Container startup completed successfully");
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
    log!("Application starting...");
    log!("Log file location: {:?}", std::env::temp_dir().join("consoley").join("app.log"));
    
    let mut app = tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            
            // 启动���器管理，但不执行清理
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

    // 仅在窗口关闭时执行清理
    app.run(|app_handle, event| {
        match event {
            RunEvent::WindowEvent { 
                label: _, 
                event: WindowEvent::CloseRequested { .. }, 
                .. 
            } => {
                log!("Window closing, cleaning up Docker resources...");
                tauri::async_runtime::block_on(async {
                    if let Err(e) = cleanup_docker().await {
                        eprintln!("Failed to cleanup Docker: {}", e);
                    }
                });
            }
            _ => {}
        }
    });
}

async fn setup_docker(app: &tauri::AppHandle) -> Result<(), String> {
    log!("Initializing Docker setup...");
    log!("Current working directory: {:?}", std::env::current_dir());
    
    let docker_manager = Arc::new(DockerManager::new().await.map_err(|e| {
        let err_msg = format!("Failed to create Docker manager: {}", e);
        log!("{}", err_msg);
        err_msg
    })?);
    
    // 确保镜像存在
    let image_tag = if cfg!(debug_assertions) {
        log!("Debug mode detected, using dev image");
        "consoleai/desktop:dev"
    } else {
        log!("Release mode detected, using latest image");
        "consoleai/desktop:latest"
    };
    
    log!("Ensuring Docker image exists: {}", image_tag);
    if let Err(e) = docker_manager.ensure_image(app, image_tag).await {
        let err_msg = format!("Failed to ensure Docker image: {:?}", e);
        log!("{}", err_msg);
        return Err(err_msg);
    }
    
    // 创建并启动容器
    log!("Creating and starting container...");
    match docker_manager.create_and_start_container().await {
        Ok(container_id) => {
            log!("Container started successfully with ID: {}", container_id);
            
            // 发送服务就绪事件
            app.emit("vnc-ready", ()).map_err(|e| e.to_string())?;
            
            Ok(())
        }
        Err(e) => {
            let err_msg = format!("Failed to start container: {:?}", e);
            log!("{}", err_msg);
            Err(err_msg)
        }
    }
}

async fn cleanup_docker() -> Result<(), String> {
    let docker_manager = Arc::new(DockerManager::new().await.map_err(|e| e.to_string())?);
    docker_manager.cleanup().await.map_err(|e| e.to_string())
}
