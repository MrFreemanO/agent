use std::path::PathBuf;
use std::process::Command;
use tokio::fs;
use std::collections::HashMap;
use std::net::TcpListener;

pub struct DockerManager {
    image_path: Option<PathBuf>,
    is_dev: bool,
}

impl DockerManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let is_dev = cfg!(debug_assertions);
        let image_path = if !is_dev {
            let app_dir = tauri::api::path::app_local_data_dir().unwrap();
            Some(app_dir.join("resources/desktop.tar"))
        } else {
            None
        };
        
        Ok(Self { image_path, is_dev })
    }

    fn get_image_tag(&self) -> String {
        if self.is_dev {
            "consoleai/desktop:dev".to_string()
        } else {
            #[cfg(target_arch = "aarch64")]
            return "consoleai/desktop:latest-arm64".to_string();
            
            #[cfg(not(target_arch = "aarch64"))]
            return "consoleai/desktop:latest".to_string();
        }
    }

    pub async fn ensure_image(&self) -> Result<(), Box<dyn std::error::Error>> {
        let image_tag = self.get_image_tag();

        // 检查镜像是否存在
        let output = Command::new("docker")
            .args(&["images", "-q", image_tag])
            .output()?;

        if output.stdout.is_empty() {
            if self.is_dev {
                // 开发环境：使用 docker build
                let dockerfile = if self.is_dev {
                    "../docker/desktop/Dockerfile.dev"
                } else {
                    "../docker/desktop/Dockerfile"
                };

                let status = Command::new("docker")
                    .args(&[
                        "build",
                        "-t",
                        image_tag,
                        "-f",
                        dockerfile,
                        "../docker/desktop",
                    ])
                    .status()?;

                if !status.success() {
                    return Err("Failed to build Docker image".into());
                }
            } else {
                // 生产环境：从嵌入的tar文件加载
                if let Some(image_path) = &self.image_path {
                    // 如果资源目录不存在，创建它
                    if let Some(parent) = image_path.parent() {
                        fs::create_dir_all(parent).await?;
                    }

                    // 复制嵌入的镜像文件
                    let embedded_image = include_bytes!(concat!(env!("OUT_DIR"), "/resources/desktop.tar"));
                    fs::write(image_path, embedded_image).await?;

                    // 导入镜像
                    let status = Command::new("docker")
                        .args(&[
                            "load",
                            "-i",
                            image_path.to_str().unwrap(),
                        ])
                        .status()?;

                    if !status.success() {
                        return Err("Failed to load Docker image".into());
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
        
        // API服务端口映射 - 修改这里
        let binding8080 = vec![PortBinding {
            host_ip: Some(String::from("0.0.0.0")),
            host_port: Some(String::from("8090")),
        }];

        port_bindings.insert(String::from("5900/tcp"), Some(binding5900));
        port_bindings.insert(String::from("6080/tcp"), Some(binding6080));
        port_bindings.insert(String::from("8080/tcp"), Some(binding8080));  // 确保这里的key是 "8080/tcp"

        // 创建容器
        let container_id = Command::new("docker")
            .args(&["run", "-d", "--name", "desktop", "-p", "5900:5800", "-p", "6080:6070", "-p", "8080:8090", self.get_image_tag()])
            .output()?
            .stdout;

        // 明确绑定到所有接口
        let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");
        log::info!("Listening on http://0.0.0.0:8080");

        Ok(String::from_utf8(container_id).unwrap())
    }
} 