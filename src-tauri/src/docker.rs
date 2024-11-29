use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions};
use bollard::models::{HostConfig, PortBinding, ContainerSummary};
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;
use std::collections::HashMap;
use std::sync::Arc;

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

    pub async fn ensure_image(&self, image: &str) -> Result<(), DockerError> {
        // 首先检查本地是否存在镜像
        let images = self.docker.list_images::<String>(None).await?;
        let image_exists = images.iter().any(|img| {
            img.repo_tags
                .iter()
                .any(|tag| tag == image)
        });

        if !image_exists {
            println!("Image not found locally, attempting to build...");
            // 构建镜像
            use bollard::image::BuildImageOptions;

            let mut path = std::env::current_dir()?;
            path.push("docker");
            path.push("desktop");

            println!("Building from path: {:?}", path);

            let options = BuildImageOptions {
                dockerfile: "Dockerfile",
                t: image,
                rm: true,
                ..Default::default()
            };

            let tar_gz = tar_directory(&path)?;
            let mut build_stream = self.docker.build_image(options, None, Some(tar_gz.into()));
            
            while let Some(result) = build_stream.next().await {
                match result {
                    Ok(output) => {
                        if let Some(stream) = output.stream {
                            print!("{}", stream);
                        }
                    }
                    Err(e) => {
                        println!("Build error: {}", e);
                        return Err(DockerError::Image(e.to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn create_and_start_container(&self) -> Result<String, DockerError> {
        // 创建容器配置
        let mut port_bindings = HashMap::new();
        
        // VNC端口映射
        let binding5900 = vec![PortBinding {
            host_ip: Some(String::from("0.0.0.0")),
            host_port: Some(String::from("5800")),
        }];
        
        // noVNC端口映射
        let binding6080 = vec![PortBinding {
            host_ip: Some(String::from("0.0.0.0")),
            host_port: Some(String::from("6070")),
        }];
        
        // API服务端口映射
        let binding8080 = vec![PortBinding {
            host_ip: Some(String::from("0.0.0.0")),
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
            image: Some(String::from("consoleai/desktop:latest")),
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

        Ok(container.id)
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

        let logs = self.docker
            .logs(container_id, Some(options))
            .try_collect::<Vec<_>>()
            .await?;

        Ok(logs.into_iter()
            .map(|log| log.to_string())
            .collect::<Vec<_>>()
            .join("\n"))
    }

    pub async fn restart_container(&self, container_id: &str) -> Result<(), DockerError> {
        self.docker
            .restart_container(container_id, None)
            .await?;
        Ok(())
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