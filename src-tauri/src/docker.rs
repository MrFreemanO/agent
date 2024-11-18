use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, StopContainerOptions};
use bollard::models::HostConfig;
use bollard::models::PortBinding;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;

pub struct DockerManager {
    docker: Docker,
}

impl DockerManager {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(DockerManager { docker })
    }

    // 加载内置的Docker镜像
    // 检查并拉取镜像
    pub async fn ensure_image(&self, image: &str) -> Result<(), Box<dyn Error>> {
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
                        return Err(Box::new(e));
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn create_and_start_container(&self) -> Result<String, Box<dyn Error>> {
        // 检查并移除已存在的容器
        match self.docker.inspect_container("consoley-desktop", None).await {
            Ok(_) => {
                println!("Found existing container, stopping and removing it...");
                let _ = self.docker.stop_container("consoley-desktop", None).await;
                let _ = self.docker.remove_container("consoley-desktop", None).await;
            }
            Err(_) => {
                println!("No existing container found, proceeding with creation...");
            }
        }
    
        let config = CreateContainerOptions {
            name: "consoley-desktop",
            platform: None,
        };
        
        let host_config = HostConfig {
            port_bindings: Some({
                let mut bindings = HashMap::new();
                let binding5900 = vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some("5800".to_string()),
                }];
                let binding6080 = vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some("6070".to_string()),
                }];
                let binding8080 = vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some("8090".to_string()),
                }];
                bindings.insert(String::from("5900/tcp"), Some(binding5900));
                bindings.insert(String::from("6080/tcp"), Some(binding6080));
                bindings.insert(String::from("8080/tcp"), Some(binding8080));
                bindings
            }),
            privileged: Some(true),
            security_opt: Some(vec![String::from("seccomp=unconfined")]),
            binds: Some(vec![String::from("/tmp/.X11-unix:/tmp/.X11-unix:rw")]),
            shm_size: Some(67108864), // 64MB
            ..Default::default()
        };
    
        // 环境变量配置
        let env = vec![
            String::from("DISPLAY=:1"),
            String::from("RESOLUTION=1280x720x24"),
            String::from("VNC_PASSWORD=consoley"),
            String::from("LANG=en_US.UTF-8"),
            String::from("LANGUAGE=en_US:en"),
            String::from("LC_ALL=en_US.UTF-8")
        ];
    
        let container = self.docker.create_container(
            Some(config),
            Config {
                image: Some(String::from("consoleai/desktop:latest")),
                hostname: Some(String::from("consoley")),
                exposed_ports: Some({
                    let mut ports = HashMap::new();
                    ports.insert("5900/tcp".to_string(), HashMap::new());
                    ports.insert("6080/tcp".to_string(), HashMap::new());
                    ports.insert("8080/tcp".to_string(), HashMap::new());
                    ports
                }),
                env: Some(env),
                host_config: Some(host_config),
                working_dir: Some(String::from("/root")),
                tty: Some(true),
                ..Default::default()
            },
        ).await?;
    
        self.docker.start_container(&container.id, None::<StartContainerOptions<String>>).await?;
        
        Ok(container.id)
    }
    


    // 停止容器
    pub async fn stop_container(&self, container_id: &str) -> Result<(), Box<dyn Error>> {
        self.docker
            .stop_container(container_id, None::<StopContainerOptions>)
            .await?;
        Ok(())
    }
    

    // 删除容器
    pub async fn remove_container(&self, container_id: &str) -> Result<(), Box<dyn Error>> {
        self.docker.remove_container(container_id, None).await?;
        Ok(())
    }

    // 检查容器状态
    pub async fn check_container_status(&self, container_id: &str) -> Result<String, Box<dyn Error>> {
        let container = self.docker.inspect_container(container_id, None).await?;
        Ok(container.state
            .and_then(|state| state.status)
            .map(|status| status.to_string())
            .unwrap_or_else(|| "unknown".to_string()))
    }

    // 添加获取容器日志的方法
    pub async fn get_container_logs(&self, container_id: &str) -> Result<String, Box<dyn Error>> {
        use bollard::container::LogsOptions;
        
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            ..Default::default()
        };

        let logs = self.docker.logs(container_id, Some(options));
        let mut output = String::new();
        
        futures_util::pin_mut!(logs);
        while let Some(log) = logs.next().await {
            match log {
                Ok(log) => output.push_str(&log.to_string()),
                Err(e) => eprintln!("Error getting logs: {}", e),
            }
        }
        
        Ok(output)
    }

    // 添加重启容器的方法
    pub async fn restart_container(&self, container_id: &str) -> Result<(), Box<dyn Error>> {
        self.docker.restart_container(container_id, None).await?;
        Ok(())
    }
}

fn tar_directory(path: &std::path::Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut tar = tar::Builder::new(Vec::new());
    tar.append_dir_all(".", path)?;
    Ok(tar.into_inner()?)
} 