use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::process::Command;
use std::fs;
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize)]
struct ScreenshotResponse {
    r#type: String,
    media_type: String,
    data: String,
}

#[get("/computer/screenshot")]
async fn take_screenshot() -> impl Responder {
    let screenshot_path = "/tmp/screenshot.png";
    
    let result = Command::new("/usr/bin/scrot")
        .arg(screenshot_path)
        .env("DISPLAY", ":1")
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                return HttpResponse::InternalServerError().json(ScreenshotResponse {
                    r#type: String::from("base64"),
                    media_type: String::from("image/png"),
                    data: String::new(),
                });
            }

            match fs::read(screenshot_path) {
                Ok(image_data) => {
                    let base64_string = general_purpose::STANDARD.encode(&image_data);
                    let _ = fs::remove_file(screenshot_path);
                    
                    HttpResponse::Ok().json(ScreenshotResponse {
                        r#type: String::from("base64"),
                        media_type: String::from("image/png"),
                        data: base64_string,
                    })
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(ScreenshotResponse {
                        r#type: String::from("base64"),
                        media_type: String::from("image/png"),
                        data: String::new(),
                    })
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ScreenshotResponse {
                r#type: String::from("base64"),
                media_type: String::from("image/png"),
                data: String::new(),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting API server on http://0.0.0.0:8090");
    
    HttpServer::new(|| {
        App::new()
            .service(take_screenshot)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
} 