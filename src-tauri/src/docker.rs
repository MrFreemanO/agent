use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions, ListContainersOptions};
use bollard::image::ListImagesOptions;
use bollard::models::{HostConfig, PortBinding, ContainerSummary};
use std::collections::HashMap;
use std::sync::Arc;
use crate::resources::extract_docker_image;
use bytes::Bytes;
use crate::log;
use futures::{StreamExt, TryStreamExt};
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

// Add error conversion implementation
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

    #[allow(dead_code)]
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

        // Check if the image exists
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

            // Read the image file
            let image_data = tokio::fs::read(&image_path).await
                .map_err(|e| DockerError::IO(format!("Failed to read image file: {}", e)))?;
            
            log!("Loading Docker image using bollard");

            // Convert the image data directly to Bytes
            let image_bytes = Bytes::from(image_data);

            // Use import_image to import the image
            self.docker.import_image(
                bollard::image::ImportImageOptions {
                    ..Default::default()
                },
                image_bytes, // Pass Bytes directly
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

    #[allow(dead_code)]
    pub async fn cleanup_existing_container(&self) -> Result<(), DockerError> {
        log!("Cleaning up existing containers...");
    
        // List all related containers (including stopped)
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
    
                // Stop the container
                let _ = self.docker
                    .stop_container(&id, None)
                    .await;
    
                // Remove the container
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
    
        // Wait for a while to ensure resources are released
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
        Ok(())
    }    

    fn get_api_server_path(&self) -> Result<PathBuf, DockerError> {
        let current_dir = std::env::current_dir()
            .map_err(|e| DockerError::IO(format!("Failed to get current directory: {}", e)))?;
            
        let project_root = current_dir
            .parent() // Go back to the project root directory
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
        // Verify api-server path
        self.verify_api_server_path().await?;
        
        log!("Checking for existing containers...");
        let containers = self.docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true, // Include stopped containers
                filters: {
                    let mut filters = HashMap::new();
                    filters.insert("name".to_string(), vec!["consoley".to_string()]);
                    filters
                },
                ..Default::default()
            }))
            .await
            .map_err(|e| DockerError::Container(e.to_string()))?;

        // If an existing container is found, try to reuse it
        if let Some(container) = containers.first() {
            if let Some(id) = &container.id {
                log!("Found existing container: {}", id);
                
                // Check container status
                if container.state.as_deref() != Some("running") {
                    log!("Starting existing container...");
                    self.docker
                        .start_container(id, None::<StartContainerOptions<String>>)
                        .await
                        .map_err(|e| DockerError::Container(e.to_string()))?;
                } else {
                    log!("Container is already running");
                }
                
                // Wait for services to be ready
                self.wait_for_services(id).await?;
                return Ok(id.clone());
            }
        }

        // Only create a new one if no existing container is found
        log!("No existing container found, creating new one...");
        
        // Create new container code
        let mut port_bindings = HashMap::new();
        
        // VNC port mapping
        let binding5900 = vec![PortBinding {
            host_ip: Some(String::from("127.0.0.1")),
            host_port: Some(String::from("5800")),
        }];
        
        // noVNC port mapping
        let binding6080 = vec![PortBinding {
            host_ip: Some(String::from("127.0.0.1")),
            host_port: Some(String::from("6070")),
        }];
        
        // API server port mapping
        let binding8080 = vec![PortBinding {
            host_ip: Some(String::from("127.0.0.1")),
            host_port: Some(String::from("8090")),
        }];

        port_bindings.insert(String::from("5900/tcp"), Some(binding5900));
        port_bindings.insert(String::from("6080/tcp"), Some(binding6080));
        port_bindings.insert(String::from("8080/tcp"), Some(binding8080));

        // Get api-server directory path
        let api_server_path = self.get_api_server_path()?;
        
        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            privileged: Some(true),
            binds: Some(vec![
                String::from("/tmp/.X11-unix:/tmp/.X11-unix:rw"),
                format!("{}:/etc/supervisor/conf.d/supervisord.conf", self.get_supervisor_config_path()?),
                // Use verified api-server path
                format!("{}:/app/api-server", api_server_path.to_string_lossy()),
            ]),
            ..Default::default()
        };

        log!("Creating container with config");

        let config = Config {
            image: Some(self.image_tag.clone()),
            host_config: Some(host_config),
            env: Some(vec![
                String::from("DISPLAY=:1"),
                String::from("WIDTH=1024"),
                String::from("HEIGHT=768"),
                String::from("RUST_LOG=debug"),
                String::from("RUST_BACKTRACE=full"),
            ]),
            ..Default::default()
        };

        let container = self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: "consoley-desktop",
                    platform: None,
                }),
                config
            )
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

        // Wait for the container to fully start and check service status
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        // Check supervisor status
        let logs = self.get_container_logs(&container.id).await?;
        if logs.contains("exited: api-server (exit status 1)") {
            return Err("API server failed to start. Check supervisor logs for details.".into());
        }

        Ok(container.id)
    }

    // Add a method to wait for services to be ready
    async fn wait_for_services(&self, container_id: &str) -> Result<(), DockerError> {
        log!("Waiting for services to be ready...");
        
        // Wait for full startup
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // Check container status
        let container_info = self.docker
            .inspect_container(container_id, None)
            .await
            .map_err(|e| DockerError::Container(e.to_string()))?;

        if !container_info.state.unwrap().running.unwrap_or(false) {
            return Err(DockerError::Container("Container is not running".to_string()));
        }

        // Wait for services to be ready
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

    // Add a public cleanup method
    pub async fn cleanup(&self) -> Result<(), DockerError> {
        // println!("Cleaning up existing containers...");
        // Cleanup logic...
        Ok(())
    }

    fn get_supervisor_config_path(&self) -> Result<String, DockerError> {
        let config_name = if cfg!(debug_assertions) {
            "supervisord.dev.conf"
        } else {
            "supervisord.conf"
        };
        
        // Go up one level from the current directory (src-tauri) to find the project root directory
        let current_dir = std::env::current_dir()
            .map_err(|e| DockerError::IO(format!("Failed to get current directory: {}", e)))?;
            
        let project_root = current_dir
            .parent() // Go back to the project root directory
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

// Implement Send and Sync
unsafe impl Send for DockerManager {}
unsafe impl Sync for DockerManager {}