use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions};
use bollard::models::{HostConfig, PortBinding, ContainerSummary};
use std::collections::HashMap;
use std::sync::Arc;
use std::process::Command;
use crate::resources::extract_docker_image;
use futures::StreamExt;

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
}

impl DockerManager {
    pub async fn new() -> Result<Self, DockerError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| DockerError::Connection(e.to_string()))?;
        Ok(DockerManager { 
            docker: Arc::new(docker) 
        })
    }

    pub async fn list_containers(&self) -> Result<Vec<ContainerSummary>, DockerError> {
        self.docker.list_containers::<String>(None)
            .await
            .map_err(|e| DockerError::Container(e.to_string()))
    }

    pub async fn ensure_image(&self, app: &tauri::AppHandle, image_tag: &str) -> Result<(), DockerError> {
        println!("Ensuring Docker image: {}", image_tag);
        
        // 首先检查并清理已存在的容器
        self.cleanup_existing_container().await?;
        
        // 检查镜像是否存在
        let output = Command::new("docker")
            .args(&["images", "-q", image_tag])
            .output()
            .map_err(|e| DockerError::IO(e.to_string()))?;

        if output.stdout.is_empty() {
            println!("Docker image {} not found, attempting to load from resources...", image_tag);
            match extract_docker_image(app).await {
                Ok(image_path) => {
                    println!("Loading Docker image from: {:?}", image_path);
                    let load_status = Command::new("docker")
                        .args(&["load", "-i", image_path.to_str().unwrap()])
                        .status()
                        .map_err(|e| DockerError::IO(e.to_string()))?;

                    if !load_status.success() {
                        return Err(DockerError::Image("Failed to load Docker image".to_string()));
                    }
                    println!("Docker image loaded successfully.");
                }
                Err(e) => return Err(DockerError::Image(e.to_string()))
            }
        }

        Ok(())
    }

    pub async fn cleanup_existing_container(&self) -> Result<(), DockerError> {
        println!("Cleaning up existing containers...");
        
        // 使用 docker ps 命令查找所有相关容器（包括停止的）
        let output = Command::new("docker")
            .args(&["ps", "-aq", "--filter", "name=consoley"])
            .output()
            .map_err(|e| DockerError::IO(e.to_string()))?;

        // 存储转换后的字符串
        let output_str = String::from_utf8_lossy(&output.stdout);
        let container_ids: Vec<&str> = output_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        for container_id in container_ids {
            println!("Removing container: {}", container_id);
            
            // 强制停止容器
            let _ = Command::new("docker")
                .args(&["kill", container_id])
                .output();

            // 强制删除容器
            let rm_status = Command::new("docker")
                .args(&["rm", "-f", container_id])
                .status()
                .map_err(|e| DockerError::IO(e.to_string()))?;

            if !rm_status.success() {
                return Err(DockerError::Container(format!("Failed to remove container {}", container_id)));
            }
        }

        // 等待一段时间确保资源释放
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(())
    }

    // 添加环境判断函数
    fn get_image_tag() -> String {
        if cfg!(debug_assertions) {
            String::from("consoleai/desktop:dev")
        } else {
            String::from("consoleai/desktop:latest")
        }
    }

    pub async fn create_and_start_container(&self) -> Result<String, DockerError> {
        // 先清理已存在的容器
        self.cleanup_existing_container().await?;
        
        // 等待一下确保端口释放
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 创建容器配置
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

        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            privileged: Some(true),
            binds: Some(vec![String::from("/tmp/.X11-unix:/tmp/.X11-unix:rw")]),
            ..Default::default()
        };

        let config = Config {
            image: Some(Self::get_image_tag()),
            host_config: Some(host_config),
            env: Some(vec![
                String::from("DISPLAY=:1"),
                String::from("WIDTH=1024"),
                String::from("HEIGHT=768"),
            ]),
            ..Default::default()
        };

        // 创建容器
        let container = self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: "consoley",
                    platform: None,
                }),
                config,
            )
            .await?;

        // 启动容器
        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await?;

        // 等待服务就绪
        self.wait_for_services(&container.id).await?;

        Ok(container.id)
    }

    // 添加等待服务就绪的方法
    async fn wait_for_services(&self, container_id: &str) -> Result<(), DockerError> {
        println!("Waiting for services to be ready...");
        
        // 等待容器完全启动
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // 检查容器状态
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
                    println!("All services are ready!");
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
        self.cleanup_existing_container().await
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