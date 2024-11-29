use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../docker/desktop/Dockerfile");
    println!("cargo:rerun-if-changed=../docker/desktop/Dockerfile.dev");
    println!("cargo:rerun-if-changed=../docker/desktop/supervisord.conf");
    println!("cargo:rerun-if-changed=../docker/desktop/startup.sh");
    println!("cargo:rerun-if-changed=../docker/desktop/startup.dev.sh");

    // 判断是否为开发环境
    let is_dev = std::env::var("CARGO_PROFILE").unwrap_or_default() == "debug";
    
    let dockerfile = if is_dev {
        "../docker/desktop/Dockerfile.dev"
    } else {
        "../docker/desktop/Dockerfile"
    };

    let image_tag = if is_dev {
        "consoleai/desktop:dev"
    } else {
        "consoleai/desktop:latest"
    };

    // 构建Docker镜像
    let docker_build = Command::new("docker")
        .args(&[
            "build",
            "-t",
            image_tag,
            "-f",
            dockerfile,
            "../docker/desktop",
        ])
        .status()
        .expect("Failed to build Docker image");

    if !docker_build.success() {
        panic!("Docker build failed");
    }

    // 导出Docker镜像
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let image_path = Path::new(&out_dir).join("desktop.tar");

    let docker_save = Command::new("docker")
        .args(&[
            "save",
            "-o",
            image_path.to_str().unwrap(),
            image_tag,
        ])
        .status()
        .expect("Failed to save Docker image");

    if !docker_save.success() {
        panic!("Docker save failed");
    }
    tauri_build::build();
}
