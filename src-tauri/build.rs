use std::env;
use std::path::Path;
use std::process::Command;
use std::fs;

fn main() {
    // 监听文件变化以重启构建
    println!("cargo:rerun-if-changed=../docker/desktop/Dockerfile");
    println!("cargo:rerun-if-changed=../docker/desktop/Dockerfile.dev");
    println!("cargo:rerun-if-changed=../docker/desktop/supervisord.conf");
    println!("cargo:rerun-if-changed=../docker/desktop/startup.sh");
    println!("cargo:rerun-if-changed=../docker/desktop/startup.dev.sh");

    // 判断构建模式
    let profile = env::var("PROFILE").unwrap_or_default();
    let is_release = profile == "release";

    if is_release {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let tauri_resources_dir = manifest_dir.join("resources");
        let target_image_path = tauri_resources_dir.join("desktop.tar");

        // 打印当前目录和目标路径信息，用于调试
        println!("Current directory: {:?}", std::env::current_dir().unwrap());
        println!("Target image path: {:?}", target_image_path);
        println!("Manifest directory: {:?}", manifest_dir);

        // 确保源目录存在
        fs::create_dir_all(&tauri_resources_dir).unwrap_or_else(|e| {
            panic!("Failed to create resources directory: {}", e);
        });

        // 如果源目录中不存在文件，则需要生成
        if !target_image_path.exists() {
            println!("Building Docker image for production...");
            let docker_build = Command::new("docker")
                .args(&[
                    "build",
                    "-t",
                    "consoleai/desktop:latest",
                    "-f",
                    "../docker/desktop/Dockerfile",
                    "../docker/desktop",
                ])
                .status()
                .expect("Failed to build Docker image");

            if !docker_build.success() {
                panic!("Docker build failed");
            }

            println!("Saving Docker image to desktop.tar...");
            let docker_save = Command::new("docker")
                .args(&[
                    "save",
                    "-o",
                    target_image_path.to_str().unwrap(),
                    "consoleai/desktop:latest",
                ])
                .status()
                .expect("Failed to save Docker image");

            if !docker_save.success() {
                panic!("Docker save failed");
            }
        } else {
            println!("desktop.tar exists at: {:?}", target_image_path);
            
            // 验证文件大小
            if let Ok(metadata) = fs::metadata(&target_image_path) {
                println!("desktop.tar size: {} bytes", metadata.len());
                if metadata.len() < 1000000 { // 假设文件至少应该有1MB
                    println!("desktop.tar seems too small, rebuilding...");
                    fs::remove_file(&target_image_path).unwrap();
                    // 重新运行构建过程...
                    // (这里可以提取构建过程为一个函数，避免代码重复)
                }
            }
        }

        // 获取构建输出目录
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
        let out_resources_dir = Path::new(&out_dir).join("resources");
        
        // 确保构建输出目录存在
        fs::create_dir_all(&out_resources_dir).unwrap_or_else(|e| {
            panic!("Failed to create output resources directory: {}", e);
        });

        // 复制到构建输出目录
        let out_image_path = out_resources_dir.join("desktop.tar");
        println!("Copying desktop.tar to build output directory: {:?}", out_image_path);
        fs::copy(&target_image_path, &out_image_path).unwrap_or_else(|e| {
            panic!("Failed to copy desktop.tar to build output directory: {}", e);
        });

        // 重要：确保文件也存在于 src-tauri/resources 目录
        println!("Ensuring desktop.tar exists in src-tauri/resources: {:?}", target_image_path);
        if !target_image_path.exists() {
            fs::copy(&out_image_path, &target_image_path).unwrap_or_else(|e| {
                panic!("Failed to copy desktop.tar to src-tauri/resources: {}", e);
            });
        }

        // 验证两个位置的文件
        let src_size = fs::metadata(&target_image_path).unwrap().len();
        let out_size = fs::metadata(&out_image_path).unwrap().len();
        println!("Source file size: {} bytes", src_size);
        println!("Output file size: {} bytes", out_size);

        assert_eq!(src_size, out_size, "File sizes don't match!");
    }

    // 调用 Tauri 的 build
    tauri_build::build()
}
