use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions, ListContainersOptions};
use bollard::image::{ListImagesOptions, CreateImageOptions, ImportImageOptions};
use bollard::models::{HostConfig, PortBinding, ContainerSummary};
use std::collections::HashMap;
use std::sync::Arc;
use crate::resources::extract_docker_image;
use bytes::Bytes;
use crate::log;
use futures::stream::{self, BoxStream};
use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;
use bollard::auth::DockerCredentials;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DockerError {
    Connection(String),
    Container(String),
    Image(String),
    IO(String),
}

impl std::error::Error for DockerError {}

impl std::fmt::Display for DockerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DockerError::Connection(msg) => write!(f, "Docker connection error: {}", msg),
            DockerError::Container(msg) => write!(f, "Container operation error: {}", msg),
            DockerError::Image(msg) => write!(f, "Image operation error: {}", msg),
            DockerError::IO(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

// 添加错误转换实现
impl From<bollard::errors::Error> for DockerError {
    fn from(err: bollard::errors::Error) -> Self {
        DockerError::Container(err.to_string())
    }
}

impl From<std::io::Error> for DockerError {
    fn from(err: std::io::Error) -> Self {
        DockerError::IO(err.to_string())
    }
}

pub struct DockerManager {
    docker: Arc<Docker>,
    image_loaded: std::sync::atomic::AtomicBool,
    image_tag: String,
}

impl DockerManager {
    pub async fn new() -> Result<Self, DockerError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| DockerError::Connection(e.to_string()))?;
        
        let image_tag = if cfg!(debug_assertions) {
            String::from("consoleai/desktop:dev")
        } else {
            String::from("consoleai/desktop:latest")
        };

        Ok(DockerManager { 
            docker: Arc::new(docker),
            image_loaded: std::sync::atomic::AtomicBool::new(false),
            image_tag,
        })
    }

    pub async fn list_containers(&self) -> Result<Vec<ContainerSummary>, DockerError> {
        self.docker.list_containers::<String>(None)
            .await
            .map_err(|e| DockerError::Container(e.to_string()))
    }
    
    pub async fn ensure_image(&self, app: &tauri::AppHandle, image_tag: &str) -> Result<(), DockerError> {
        if self.image_loaded.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }

        log!("Ensuring Docker image: {}", image_tag);

        // 检查镜像是否存在
        let images = self.docker
            .list_images(Some(ListImagesOptions::<String> {
                all: true,
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("reference".to_string(), vec![image_tag.to_string()]);
                    filters
                },
                ..Default::default()
            }))
            .await
            .map_err(|e| DockerError::Image(e.to_string()))?;

        if images.is_empty() {
            log!("Docker image {} not found, attempting to load from resources...", image_tag);

            let image_path = extract_docker_image(app)
                .await
                .map_err(|e| DockerError::IO(e.to_string()))?;
            
            log!("Loading Docker image from: {:?}", image_path);

            // 读取镜像文件
            let image_data = tokio::fs::read(&image_path).await
                .map_err(|e| DockerError::IO(format!("Failed to read image file: {}", e)))?;
            
            log!("Loading Docker image using bollard");

            // 将镜像数据直接转换为 Bytes
            let image_bytes = Bytes::from(image_data);

            // 使用 import_image 导入镜像
            self.docker.import_image(
                bollard::image::ImportImageOptions {
                    ..Default::default()
                },
                image_bytes, // 直接传递 Bytes
                None,
            )
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| DockerError::Image(e.to_string()))?;

            log!("Docker image loaded successfully using bollard");
        }

        self.image_loaded.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub async fn cleanup_existing_container(&self) -> Result<(), DockerError> {
        log!("Cleaning up existing containers...");
    
        // 列出所有相关容器（包括停止的）
        let containers = self.docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("name".to_string(), vec!["consoley".to_string()]);
                    filters
                },
                ..Default::default()
            }))
            .await
            .map_err(|e| DockerError::Container(e.to_string()))?;
    
        for container in containers {
            if let Some(id) = container.id {
                log!("Removing container: {}", id);
    
                // 停止容器
                let _ = self.docker
                    .stop_container(&id, None)
                    .await;
    
                // 删除容器
                self.docker
                    .remove_container(
                        &id,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await
                    .map_err(|e| DockerError::Container(e.to_string()))?;
            }
        }
    
        // 等待一段时间确保资源释放
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
        Ok(())
    }    

    fn get_api_server_path(&self) -> Result<PathBuf, DockerError> {
        let current_dir = std::env::current_dir()
            .map_err(|e| DockerError::IO(format!("Failed to get current directory: {}", e)))?;
            
        let project_root = current_dir
            .parent() // 回到项目根目录
            .ok_or_else(|| DockerError::IO("Failed to get project root directory".to_string()))?;
            
        let api_server_path = project_root
            .join("docker")
            .join("desktop")
            .join("api-server");
            
        log!("Looking for api-server at: {}", api_server_path.display());
            
        if !api_server_path.exists() {
            return Err(DockerError::IO(format!(
                "api-server directory not found at: {}",
                api_server_path.display()
            )));
        }

        if !api_server_path.join("Cargo.toml").exists() {
            return Err(DockerError::IO(format!(
                "Cargo.toml not found in api-server directory: {}",
                api_server_path.display()
            )));
        }
        
        Ok(api_server_path)
    }

    async fn verify_api_server_path(&self) -> Result<(), DockerError> {
        let api_server_path = self.get_api_server_path()?;
        log!("Verified api-server path at: {}", api_server_path.display());
        Ok(())
    }

    pub async fn create_and_start_container(&self) -> Result<String, Box<dyn std::error::Error>> {
        // 验证 api-server 路径
        self.verify_api_server_path().await?;
        
        log!("Checking for existing containers...");
        let containers = self.docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true, // 包括停止的容器
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("name".to_string(), vec!["consoley".to_string()]);
                    filters
                },
                ..Default::default()
            }))
            .await
            .map_err(|e| DockerError::Container(e.to_string()))?;

        // 如果找到已存在的容器，尝试复用
        if let Some(container) = containers.first() {
            if let Some(id) = &container.id {
                log!("Found existing container: {}", id);
                
                // 检查容器状态
                if container.state.as_deref() != Some("running") {
                    log!("Starting existing container...");
                    self.docker
                        .start_container(id, None::<StartContainerOptions<String>>)
                        .await
                        .map_err(|e| DockerError::Container(e.to_string()))?;
                } else {
                    log!("Container is already running");
                }
                
                // 等待服务就绪
                self.wait_for_services(id).await?;
                return Ok(id.clone());
            }
        }

        // 只有在没有找到现有容器时才创建新的
        log!("No existing container found, creating new one...");
        
        // 创建新容器的代码
        let mut port_bindings = HashMap::new();
        
        // VNC端口映射
        let binding5900 = vec![PortBinding {
            host_ip: Some(String::from("127.0.0.1")),
            host_port: Some(String::from("5800")),
        }];
        
        // noVNC端口映射
        let binding6080 = vec![PortBinding {
            host_ip: Some(String::from("127.0.0.1")),
            host_port: Some(String::from("6070")),
        }];
        
        // API服务端口映射
        let binding8080 = vec![PortBinding {
            host_ip: Some(String::from("127.0.0.1")),
            host_port: Some(String::from("8090")),
        }];

        port_bindings.insert(String::from("5900/tcp"), Some(binding5900));
        port_bindings.insert(String::from("6080/tcp"), Some(binding6080));
        port_bindings.insert(String::from("8080/tcp"), Some(binding8080));

        // 获取 api-server 目录路径
        let api_server_path = self.get_api_server_path()?;
        
        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            privileged: Some(true),
            binds: Some(vec![
                String::from("/tmp/.X11-unix:/tmp/.X11-unix:rw"),
                format!("{}:/etc/supervisor/conf.d/supervisord.conf", self.get_supervisor_config_path()?),
                // 使用验证过的 api-server 路径
                format!("{}:/app/api-server", api_server_path.to_string_lossy()),
            ]),
            ..Default::default()
        };

        log!("Creating container with config: {:?}", host_config);

        let config = Config {
            image: Some(self.image_tag.clone()),
            host_config: Some(host_config),
            env: Some(vec![
                String::from("DISPLAY=:1"),
                String::from("WIDTH=1024"),
                String::from("HEIGHT=768"),
                String::from("RUST_LOG=debug"),
                String::from("RUST_BACKTRACE=full"),
                // Add other necessary environment variables here
            ]),
            ..Default::default()
        };

        let container = self.docker
            .create_container(None::<CreateContainerOptions<String>>, config)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to create container: {}", e);
                log!("{}", err_msg);
                DockerError::Container(err_msg)
            })?;

        log!("Container created with ID: {}", container.id);

        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to start container: {}", e);
                log!("{}", err_msg);
                DockerError::Container(err_msg)
            })?;

        log!("Container started successfully");

        // 等待容器完全启动并检查服务状态
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        // 检查 supervisor 状态
        let logs = self.get_container_logs(&container.id).await?;
        if logs.contains("exited: api-server (exit status 1)") {
            return Err("API server failed to start. Check supervisor logs for details.".into());
        }

        Ok(container.id)
    }

    // 加待服务就绪的方法
    async fn wait_for_services(&self, container_id: &str) -> Result<(), DockerError> {
        log!("Waiting for services to be ready...");
        
        // 等待完全启动
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // 检容器状态
        let container_info = self.docker
            .inspect_container(container_id, None)
            .await
            .map_err(|e| DockerError::Container(e.to_string()))?;

        if !container_info.state.unwrap().running.unwrap_or(false) {
            return Err(DockerError::Container("Container is not running".to_string()));
        }

        // 等待服务就绪
        let mut retries = 0;
        const MAX_RETRIES: i32 = 30;
        
        while retries < MAX_RETRIES {
            if let Ok(logs) = self.get_container_logs(container_id).await {
                if logs.contains("success: x11vnc entered RUNNING state") {
                    log!("All services are ready!");
                    return Ok(());
                }
            }
            
            retries += 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        Err(DockerError::Container("Services failed to start within timeout".to_string()))
    }

    pub async fn stop_container(&self, container_id: &str) -> Result<(), DockerError> {
        self.docker
            .stop_container(
                container_id,
                None::<StopContainerOptions>,
            )
            .await?;
        Ok(())
    }

    pub async fn get_container_logs(&self, container_id: &str) -> Result<String, DockerError> {
        use bollard::container::LogsOptions;

        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            ..Default::default()
        };

        let mut logs = String::new();
        let mut stream = self.docker.logs(container_id, Some(options));

        while let Some(result) = stream.next().await {
            match result {
                Ok(log_output) => {
                    logs.push_str(&log_output.to_string());
                    logs.push('\n');
                }
                Err(e) => {
                    return Err(DockerError::Container(e.to_string()));
                }
            }
        }
        Ok(logs)
    }

    /// Restarts a Docker container by its ID.
    pub async fn restart_container(&self, container_id: &str) -> Result<(), DockerError> {
        self.docker
            .restart_container(container_id, None)
            .await
            .map_err(|e| DockerError::Container(e.to_string()))?;
        Ok(())
    }

    // 添加一个公共的清理方法
    pub async fn cleanup(&self) -> Result<(), DockerError> {
        // println!("Cleaning up existing containers...");
        // 清理逻辑...
        Ok(())
    }

    fn get_supervisor_config_path(&self) -> Result<String, DockerError> {
        let config_name = if cfg!(debug_assertions) {
            "supervisord.dev.conf"
        } else {
            "supervisord.conf"
        };
        
        // 从当前目录（src-tauri）向上一级找到项目根目录
        let current_dir = std::env::current_dir()
            .map_err(|e| DockerError::IO(format!("Failed to get current directory: {}", e)))?;
            
        let project_root = current_dir
            .parent() // 回到项目根目录
            .ok_or_else(|| DockerError::IO("Failed to get project root directory".to_string()))?;
            
        let config_path = project_root
            .join("docker")
            .join("desktop")
            .join(config_name);
            
        log!("Looking for supervisor config at: {}", config_path.display());
            
        if !config_path.exists() {
            return Err(DockerError::IO(format!(
                "Supervisor config file not found at: {}",
                config_path.display()
            )));
        }
        
        Ok(config_path.to_string_lossy().to_string())
    }

}

// 实现 Send 和 Sync
unsafe impl Send for DockerManager {}
unsafe impl Sync for DockerManager {}

fn tar_directory(path: &std::path::Path) -> Result<Vec<u8>, DockerError> {
    let mut tar = tar::Builder::new(Vec::new());
    tar.append_dir_all(".", path)?;
    Ok(tar.into_inner()?)
} 