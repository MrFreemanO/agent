use std::process::Command;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../docker/desktop/Dockerfile");
    println!("cargo:rerun-if-changed=../docker/desktop/supervisord.conf");
    println!("cargo:rerun-if-changed=../docker/desktop/startup.sh");

    // 构建Docker镜像
    let docker_build = Command::new("docker")
        .args(&["build", "-t", "consoleai/desktop:latest", "../docker/desktop"])
        .status()
        .expect("Failed to build Docker image");

    if !docker_build.success() {
        panic!("Docker build failed");
    }

    // 导出Docker镜像
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let image_path = Path::new(&out_dir).join("desktop.tar");
    
    let docker_save = Command::new("docker")
        .args(&["save", "-o", image_path.to_str().unwrap(), "consoleai/desktop:latest"])
        .status()
        .expect("Failed to save Docker image");

    if !docker_save.success() {
        panic!("Docker save failed");
    }
}
