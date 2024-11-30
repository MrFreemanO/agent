use std::process::Command;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use crate::resources;

#[derive(Debug)]
pub struct DockerError(pub String);

impl From<std::io::Error> for DockerError {
    fn from(err: std::io::Error) -> Self {
        DockerError(err.to_string())
    }
}

pub struct DockerManager {
    container_name: String,
    image_tag: String,
}

impl DockerManager {
    pub fn new() -> Self {
        DockerManager {
            container_name: "consoley_desktop".to_string(),
            image_tag: "consoleai/desktop:latest".to_string(),
        }
    }

    // 检查 Docker 是否运行
    pub async fn check_docker_running(&self) -> Result<bool, DockerError> {
        let output = Command::new("docker")
            .args(["info"])
            .output()?;
        
        Ok(output.status.success())
    }

    // 检查镜像是否存在
    pub async fn check_image_exists(&self) -> Result<bool, DockerError> {
        let output = Command::new("docker")
            .args(["images", "-q", &self.image_tag])
            .output()?;

        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    }

    // 加载 Docker 镜像
    pub async fn load_image(&self) -> Result<(), DockerError> {
        println!("Extracting Docker image...");
        let image_path = resources::extract_docker_image().await
            .map_err(|e| DockerError(e.to_string()))?;

        println!("Loading Docker image from {:?}...", image_path);
        let status = Command::new("docker")
            .args(["load", "-i", image_path.to_str().unwrap()])
            .status()?;

        if !status.success() {
            return Err(DockerError("Failed to load Docker image".to_string()));
        }

        Ok(())
    }

    // 检查容器是否存在
    pub async fn check_container_exists(&self) -> Result<bool, DockerError> {
        let output = Command::new("docker")
            .args(["ps", "-a", "-q", "-f", &format!("name={}", self.container_name)])
            .output()?;

        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    }

    // 创建容器
    pub async fn create_container(&self) -> Result<(), DockerError> {
        println!("Creating Docker container...");
        let status = Command::new("docker")
            .args([
                "create",
                "--name", &self.container_name,
                "-p", "5800:5900",  // VNC
                "-p", "6070:6080",  // noVNC
                "-p", "8090:8080",  // API server
                "--privileged",
                "-v", "/tmp/.X11-unix:/tmp/.X11-unix:rw",
                &self.image_tag,
            ])
            .status()?;

        if !status.success() {
            return Err(DockerError("Failed to create container".to_string()));
        }

        Ok(())
    }

    // 启动容器
    pub async fn start_container(&self) -> Result<(), DockerError> {
        println!("Starting Docker container...");
        let status = Command::new("docker")
            .args(["start", &self.container_name])
            .status()?;

        if !status.success() {
            return Err(DockerError("Failed to start container".to_string()));
        }

        // 等待容器完全启动
        sleep(Duration::from_secs(2)).await;

        Ok(())
    }

    // 停止容器
    pub async fn stop_container(&self) -> Result<(), DockerError> {
        println!("Stopping Docker container...");
        let status = Command::new("docker")
            .args(["stop", &self.container_name])
            .status()?;

        if !status.success() {
            return Err(DockerError("Failed to stop container".to_string()));
        }

        Ok(())
    }

    // 删除容器
    pub async fn remove_container(&self) -> Result<(), DockerError> {
        println!("Removing Docker container...");
        let status = Command::new("docker")
            .args(["rm", "-f", &self.container_name])
            .status()?;

        if !status.success() {
            return Err(DockerError("Failed to remove container".to_string()));
        }

        Ok(())
    }

    // 确保容器运行
    pub async fn ensure_container_running(&self) -> Result<(), DockerError> {
        // 检查 Docker 是否运行
        if !self.check_docker_running().await? {
            return Err(DockerError("Docker is not running".to_string()));
        }

        // 检查镜像是否存在，不存在则加载
        if !self.check_image_exists().await? {
            self.load_image().await?;
        }

        // 检查容器是否存在，存在则删除
        if self.check_container_exists().await? {
            self.remove_container().await?;
        }

        // 创建并启动容器
        self.create_container().await?;
        self.start_container().await?;

        Ok(())
    }

    // 获取容器状态
    pub async fn get_container_status(&self) -> Result<String, DockerError> {
        let output = Command::new("docker")
            .args(["inspect", "-f", "{{.State.Status}}", &self.container_name])
            .output()?;

        if !output.status.success() {
            return Ok("not_found".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
} 