use std::path::PathBuf;
use tokio::fs;
use tauri::Manager;
use tokio::io::AsyncReadExt;

pub async fn extract_docker_image(app: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_local_data_dir()
        .expect("Failed to get app data directory");
    
    let image_dir = app_dir.join("resources");
    let image_path = image_dir.join("desktop.tar");

    // If the image file already exists, verify its integrity
    if image_path.exists() {
        if let Ok(metadata) = fs::metadata(&image_path).await {
            // Check if the file size is reasonable (e.g., at least 100MB)
            if metadata.len() > 100_000_000 {
                // Verify if the file header is a valid tar file
                let mut file = fs::File::open(&image_path).await?;
                let mut buffer = [0u8; 512];
                if file.read_exact(&mut buffer).await.is_ok() {
                    if buffer.starts_with(&[0x1f, 0x8b]) || // gzip
                       buffer.starts_with(b"ustar") {       // tar
                        return Ok(image_path);
                    }
                }
            }
            // If verification fails, delete the file
            fs::remove_file(&image_path).await?;
        }
    }

    // Create the resources directory
    fs::create_dir_all(&image_dir).await?;

    // Read desktop.tar from the application resource directory
    let resource_dir = app.path().resource_dir()
        .expect("Failed to get resource directory");

    // Modify here: check the resources subdirectory
    let bundled_image_path = resource_dir.join("resources").join("desktop.tar");
    println!("Looking for desktop.tar in: {:?}", bundled_image_path);

    if !bundled_image_path.exists() {
        return Err(format!("Bundled desktop.tar not found at {:?}", bundled_image_path).into());
    }

    // Read and write the file
    println!("Copying desktop.tar to application directory...");
    let mut bundled_image = fs::File::open(&bundled_image_path).await?;
    let mut buffer = Vec::new();
    bundled_image.read_to_end(&mut buffer).await?;
    fs::write(&image_path, &buffer).await?;

    println!("Successfully extracted desktop.tar to: {:?}", image_path);
    Ok(image_path)
} 