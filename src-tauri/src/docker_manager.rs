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

    // Check if Docker is running
    pub async fn check_docker_running(&self) -> Result<bool, DockerError> {
        let output = Command::new("docker")
            .args(["info"])
            .output()?;
        
        Ok(output.status.success())
    }

    // Check if image exists
    pub async fn check_image_exists(&self) -> Result<bool, DockerError> {
        let output = Command::new("docker")
            .args(["images", "-q", &self.image_tag])
            .output()?;

        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    }

    // Load Docker image
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

    // Check if container exists
    pub async fn check_container_exists(&self) -> Result<bool, DockerError> {
        let output = Command::new("docker")
            .args(["ps", "-a", "-q", "-f", &format!("name={}", self.container_name)])
            .output()?;

        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    }

    // Create container
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

    // Start container
    pub async fn start_container(&self) -> Result<(), DockerError> {
        println!("Starting Docker container...");
        let status = Command::new("docker")
            .args(["start", &self.container_name])
            .status()?;

        if !status.success() {
            return Err(DockerError("Failed to start container".to_string()));
        }

        // Wait for container to fully start
        sleep(Duration::from_secs(2)).await;

        Ok(())
    }

    // Stop container
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

    // Remove container
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

    // Ensure container is running
    pub async fn ensure_container_running(&self) -> Result<(), DockerError> {
        // Check if Docker is running
        if !self.check_docker_running().await? {
            return Err(DockerError("Docker is not running".to_string()));
        }

        // Check if image exists, load if not
        if !self.check_image_exists().await? {
            self.load_image().await?;
        }

        // Check if container exists, remove if exists
        if self.check_container_exists().await? {
            self.remove_container().await?;
        }

        // Create and start container
        self.create_container().await?;
        self.start_container().await?;

        Ok(())
    }

    // Get container status
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