// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod docker;
mod resources;

use docker::DockerManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri;
use tauri::{RunEvent, WindowEvent, Emitter};
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
    let log_dir = std::env::temp_dir().join("consoley");
    let log_path = log_dir.join("app.log");
    
    // 打印当前尝试写入的日志路径
    println!("Attempting to write log to: {:?}", log_path);
    
    // 确保日志目录存在
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
        return;
    }
    
    // 尝试打开或创建日志文件
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path) 
    {
        Ok(mut file) => {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            if let Err(e) = writeln!(file, "[{}] {}", timestamp, msg) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to open log file: {}", e);
        }
    }
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
    println!("Application starting...");
    let log_path = std::env::temp_dir().join("consoley").join("app.log");
    println!("Log file will be created at: {:?}", log_path);
    
    // 尝试创建日志目录
    if let Err(e) = std::fs::create_dir_all(log_path.parent().unwrap()) {
        eprintln!("Failed to create log directory: {}", e);
    }
    
    log!("Application initialized");
    
    let mut app = tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            
            // 启动器管理，但不执行清理
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
