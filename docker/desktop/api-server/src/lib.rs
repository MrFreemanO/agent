use actix_web::{web, App, HttpServer, HttpResponse, Responder, get, post};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use actix_web::middleware::Logger;
use tokio::time::{timeout, sleep, Duration};
use std::{io::Write, process::Stdio};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::sync::Mutex;
use std::sync::OnceLock;

static BASH_SESSION: OnceLock<Mutex<Option<BashSession>>> = OnceLock::new();

#[derive(Serialize)]
pub struct ActionResponse {
    pub r#type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputerAction {
    Key,
    Type,
    MouseMove,
    LeftClick,
    LeftClickDrag,
    RightClick,
    MiddleClick,
    DoubleClick,
    Screenshot,
    CursorPosition,
}

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    pub text: Option<String>,
    pub coordinate: Option<Vec<i32>>,
}

impl ActionRequest {
    fn parse_action(&self) -> Option<ComputerAction> {
        println!("Parsing action: {}", self.action);
        
        match self.action.as_str() {
            "cursor_position" => Some(ComputerAction::CursorPosition),
            "key" => Some(ComputerAction::Key),
            "type" => Some(ComputerAction::Type),
            "mouse_move" => Some(ComputerAction::MouseMove),
            "left_click" => Some(ComputerAction::LeftClick),
            "left_click_drag" => Some(ComputerAction::LeftClickDrag),
            "right_click" => Some(ComputerAction::RightClick),
            "middle_click" => Some(ComputerAction::MiddleClick),
            "double_click" => Some(ComputerAction::DoubleClick),
            "screenshot" => Some(ComputerAction::Screenshot),
            _ => None,
        }
    }
}

fn get_cursor_position() -> Result<(i32, i32), String> {
    println!("Getting cursor position...");
    let output = Command::new("xdotool")
        .env("DISPLAY", ":1")
        .args(&["getmouselocation", "--shell"])
        .output()
        .map_err(|e| {
            println!("Failed to execute getmouselocation: {}", e);
            e.to_string()
        })?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        println!("getmouselocation command failed: {}", error);
        return Err(error);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Raw cursor position output: {}", output_str);
    
    let x = output_str.lines()
        .find(|line| line.starts_with("X="))
        .and_then(|line| {
            let value = line.trim_start_matches("X=");
            println!("Parsing X value: {}", value);
            value.parse::<i32>().ok()
        })
        .ok_or_else(|| "Failed to parse X coordinate".to_string())?;
    
    let y = output_str.lines()
        .find(|line| line.starts_with("Y="))
        .and_then(|line| {
            let value = line.trim_start_matches("Y=");
            println!("Parsing Y value: {}", value);
            value.parse::<i32>().ok()
        })
        .ok_or_else(|| "Failed to parse Y coordinate".to_string())?;

    println!("Current cursor position: ({}, {})", x, y);
    Ok((x, y))
}

pub async fn handle_computer_action(req: web::Json<ActionRequest>) -> impl Responder {
    log::info!("Processing computer action: {:?}", req);
    
    let result = match req.parse_action() {
        Some(action) => {
            match action {
                ComputerAction::Screenshot => {
                    log::info!("Executing screenshot action");
                    take_screenshot()
                },
                ComputerAction::CursorPosition => {
                    match get_cursor_position() {
                        Ok((x, y)) => HttpResponse::Ok().json(ActionResponse {
                            r#type: String::from("success"),
                            media_type: String::from("text/plain"),
                            data: format!("Cursor position is: X={}, Y={}", x, y),
                        }),
                        Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: format!("Failed to get cursor position: {}", e),
                        })
                    }
                },
                ComputerAction::Key => {
                    // 处理按键操作
                    if let Some(text) = &req.text {
                        match execute_xdotool(&["key", text]) {
                            Ok(_) => HttpResponse::Ok().json(ActionResponse {
                                r#type: String::from("success"),
                                media_type: String::from("text/plain"),
                                data: String::from("Key action executed successfully"),
                            }),
                            Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                r#type: String::from("error"),
                                media_type: String::from("text/plain"),
                                data: format!("Failed to execute key action: {}", e),
                            })
                        }
                    } else {
                        HttpResponse::BadRequest().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: String::from("Text parameter is required for key action"),
                        })
                    }
                },
                ComputerAction::Type => {
                    // 处理输入文本操作
                    if let Some(text) = &req.text {
                        match execute_xdotool(&["type", text]) {
                            Ok(_) => HttpResponse::Ok().json(ActionResponse {
                                r#type: String::from("success"),
                                media_type: String::from("text/plain"),
                                data: String::from("Type action executed successfully"),
                            }),
                            Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                r#type: String::from("error"),
                                media_type: String::from("text/plain"),
                                data: format!("Failed to execute type action: {}", e),
                            })
                        }
                    } else {
                        HttpResponse::BadRequest().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: String::from("Text parameter is required for type action"),
                        })
                    }
                },
                ComputerAction::MouseMove => {
                    // 处理鼠标移动操作
                    if let Some(coords) = &req.coordinate {
                        if coords.len() == 2 {
                            match execute_xdotool(&["mousemove", &coords[0].to_string(), &coords[1].to_string()]) {
                                Ok(_) => HttpResponse::Ok().json(ActionResponse {
                                    r#type: String::from("success"),
                                    media_type: String::from("text/plain"),
                                    data: String::from("Mouse move executed successfully"),
                                }),
                                Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                    r#type: String::from("error"),
                                    media_type: String::from("text/plain"),
                                    data: format!("Failed to execute mouse move: {}", e),
                                })
                            }
                        } else {
                            HttpResponse::BadRequest().json(ActionResponse {
                                r#type: String::from("error"),
                                media_type: String::from("text/plain"),
                                data: String::from("Coordinate must contain exactly 2 values"),
                            })
                        }
                    } else {
                        HttpResponse::BadRequest().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: String::from("Coordinate parameter is required for mouse move"),
                        })
                    }
                },
                ComputerAction::LeftClick => {
                    match execute_xdotool(&["click", "1"]) {
                        Ok(_) => HttpResponse::Ok().json(ActionResponse {
                            r#type: String::from("success"),
                            media_type: String::from("text/plain"),
                            data: String::from("Left click executed successfully"),
                        }),
                        Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: format!("Failed to execute left click: {}", e),
                        })
                    }
                },
                ComputerAction::LeftClickDrag => {
                    match execute_xdotool(&["mousedown", "1"]) {
                        Ok(_) => HttpResponse::Ok().json(ActionResponse {
                            r#type: String::from("success"),
                            media_type: String::from("text/plain"),
                            data: String::from("Left click drag executed successfully!"),
                        }),
                        Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: format!("Failed to execute left click drag: {}", e),
                        })
                    }
                },
                ComputerAction::RightClick => {
                    match execute_xdotool(&["click", "3"]) {
                        Ok(_) => HttpResponse::Ok().json(ActionResponse {
                            r#type: String::from("success"),
                            media_type: String::from("text/plain"),
                            data: String::from("Right click executed successfully"),
                        }),
                        Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: format!("Failed to execute right click: {}", e),
                        })
                    }
                },
                ComputerAction::MiddleClick => {
                    match execute_xdotool(&["click", "2"]) {
                        Ok(_) => HttpResponse::Ok().json(ActionResponse {
                            r#type: String::from("success"),
                            media_type: String::from("text/plain"),
                            data: String::from("Middle click executed successfully"),
                        }),
                        Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: format!("Failed to execute middle click: {}", e),
                        })
                    }
                },
                ComputerAction::DoubleClick => {
                    match execute_xdotool(&["click", "--repeat", "2", "1"]) {
                        Ok(_) => HttpResponse::Ok().json(ActionResponse {
                            r#type: String::from("success"),
                            media_type: String::from("text/plain"),
                            data: String::from("Double click executed successfully"),
                        }),
                        Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: format!("Failed to execute double click: {}", e),
                        })
                    }
                },
            }
        },
        None => {
            log::error!("Invalid action: {}", req.action);
            HttpResponse::BadRequest().json(ActionResponse {
                r#type: String::from("error"),
                media_type: String::from("text/plain"),
                data: String::from("Invalid action"),
            })
        }
    };
    
    log::info!("Computer action completed");
    result
}

fn take_screenshot() -> HttpResponse {
    log::info!("Taking screenshot...");
    let screenshot_path = "/tmp/screenshot.png";
    
    // 执行截图命令
    let result = Command::new("scrot")
        .arg(screenshot_path)
        .env("DISPLAY", ":1")
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                log::error!("Screenshot command failed: {}", error);
                return HttpResponse::InternalServerError().json(ActionResponse {
                    r#type: String::from("error"),
                    media_type: String::from("text/plain"),
                    data: format!("Failed to take screenshot: {}", error),
                });
            }

            // 读取截图文件
            match fs::read(screenshot_path) {
                Ok(image_data) => {
                    // 转换为 base64
                    let base64_string = general_purpose::STANDARD.encode(&image_data);
                    
                    // 清理临时文件
                    if let Err(e) = fs::remove_file(screenshot_path) {
                        log::warn!("Failed to remove temporary screenshot file: {}", e);
                    }
                    
                    log::info!("Screenshot taken successfully");
                    HttpResponse::Ok()
                        .content_type("application/json")
                        .json(ActionResponse {
                            r#type: String::from("base64"),
                            media_type: String::from("image/png"),
                            data: base64_string,
                        })
                }
                Err(e) => {
                    log::error!("Failed to read screenshot file: {}", e);
                    HttpResponse::InternalServerError().json(ActionResponse {
                        r#type: String::from("error"),
                        media_type: String::from("text/plain"),
                        data: format!("Failed to read screenshot file: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            log::error!("Failed to execute screenshot command: {}", e);
            HttpResponse::InternalServerError().json(ActionResponse {
                r#type: String::from("error"),
                media_type: String::from("text/plain"),
                data: format!("Failed to execute screenshot command: {}", e),
            })
        }
    }
}

fn execute_xdotool(args: &[&str]) -> Result<String, String> {
    println!("Executing xdotool with args: {:?}", args);
    let start = std::time::Instant::now();
    
    let result = Command::new("xdotool")
        .env("DISPLAY", ":1")
        .args(args)
        .output()
        .map_err(|e| {
            println!("xdotool command failed: {}", e);
            e.to_string()
        })
        .and_then(|output| {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                println!("xdotool succeeded with output: {}", stdout);
                Ok(stdout)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                println!("xdotool failed with error: {}", stderr);
                Err(stderr)
            }
        });
    
    println!("xdotool execution took: {:?}", start.elapsed());
    result
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditCommand {
    View,
    Create,
    StrReplace,
    Insert,
    UndoEdit,
}

#[derive(Debug, Deserialize)]
pub struct EditRequest {
    pub command: String,
    pub path: String,
    pub file_text: Option<String>,
    pub view_range: Option<Vec<i32>>,
    pub old_str: Option<String>,
    pub new_str: Option<String>,
    pub insert_line: Option<i32>,
}

impl EditRequest {
    fn parse_command(&self) -> Option<EditCommand> {
        match self.command.as_str() {
            "view" => Some(EditCommand::View),
            "create" => Some(EditCommand::Create),
            "str_replace" => Some(EditCommand::StrReplace),
            "insert" => Some(EditCommand::Insert),
            "undo_edit" => Some(EditCommand::UndoEdit),
            _ => None,
        }
    }
}

pub async fn handle_edit_action(req: web::Json<EditRequest>) -> impl Responder {
    println!("Received edit command request: {:?}", req);
    let start = std::time::Instant::now();
    
    let response = match req.parse_command() {
        Some(command) => {
            println!("Parsed command: {:?}", command);
            match command {
                EditCommand::View => {
                    println!("Processing view action");
                    match fs::read_to_string(&req.path) {
                        Ok(content) => {
                            let mut file_content = content;
                            
                            // Handle view_range if present
                            if let Some(range) = &req.view_range {
                                // First check array length
                                if range.len() != 2 {
                                    return HttpResponse::BadRequest().json(ActionResponse {
                                        r#type: String::from("error"),
                                        media_type: String::from("text/plain"),
                                        data: String::from("view_range should contain exactly 2 integers"),
                                    });
                                }

                                let lines: Vec<&str> = file_content.split('\n').collect();
                                let n_lines_file = lines.len();
                                let init_line = range[0];
                                let final_line = range[1];

                                // Validate init_line
                                if init_line < 1 || init_line as usize > n_lines_file {
                                    return HttpResponse::BadRequest().json(ActionResponse {
                                        r#type: String::from("error"),
                                        media_type: String::from("text/plain"),
                                        data: format!("Invalid view_range: first element {} should be within range [1, {}]", 
                                            init_line, n_lines_file),
                                    });
                                }

                                // Validate final_line
                                if final_line != -1 {
                                    if final_line as usize > n_lines_file {
                                        return HttpResponse::BadRequest().json(ActionResponse {
                                            r#type: String::from("error"),
                                            media_type: String::from("text/plain"),
                                            data: format!("Invalid view_range: second element {} should be smaller than number of lines {}", 
                                                final_line, n_lines_file),
                                        });
                                    }
                                    if final_line < init_line {
                                        return HttpResponse::BadRequest().json(ActionResponse {
                                            r#type: String::from("error"),
                                            media_type: String::from("text/plain"),
                                            data: format!("Invalid view_range: second element {} should be larger or equal to first element {}", 
                                                final_line, init_line),
                                        });
                                    }
                                }

                                // Extract the requested range
                                let selected_lines = if final_line == -1 {
                                    &lines[(init_line - 1) as usize..]
                                } else {
                                    &lines[(init_line - 1) as usize..final_line as usize]
                                };
                                
                                file_content = selected_lines.join("\n");
                            }

                            HttpResponse::Ok().json(ActionResponse {
                                r#type: String::from("success"),
                                media_type: String::from("text/plain"),
                                data: file_content,
                            })
                        },
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                HttpResponse::BadRequest().json(ActionResponse {
                                    r#type: String::from("error"),
                                    media_type: String::from("text/plain"),
                                    data: format!("File not found: {}", req.path),
                                })
                            } else {
                                HttpResponse::InternalServerError().json(ActionResponse {
                                    r#type: String::from("error"),
                                    media_type: String::from("text/plain"),
                                    data: format!("Failed to read file: {}", e),
                                })
                            }
                        }
                    }
                },
                EditCommand::Create => {
                    println!("Processing create action");
                    if let Some(text) = &req.file_text {
                        println!("Creating file at: {} with content length: {}", req.path, text.len());
                        match fs::write(&req.path, text) {
                            Ok(_) => {
                                println!("File created successfully");
                                HttpResponse::Ok().json(ActionResponse {
                                    r#type: String::from("success"),
                                    media_type: String::from("text/plain"),
                                    data: format!("File created successfully at: {}", req.path),
                                })
                            },
                            Err(e) => {
                                println!("Failed to create file: {}", e);
                                HttpResponse::InternalServerError().json(ActionResponse {
                                    r#type: String::from("error"),
                                    media_type: String::from("text/plain"),
                                    data: format!("Failed to create file: {}", e),
                                })
                            }
                        }
                    } else {
                        println!("Missing file_text parameter");
                        HttpResponse::BadRequest().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: String::from("file_text is required for create action"),
                        })
                    }
                },
                EditCommand::StrReplace => {
                    println!("Processing string replace action");
                    if let (Some(old_str), Some(new_str)) = (&req.old_str, &req.new_str) {
                        match fs::read_to_string(&req.path) {
                            Ok(content) => {
                                let new_content = content.replace(old_str, new_str);
                                // 创建备份文件
                                let backup_path = format!("{}.bak", req.path);
                                if let Err(e) = fs::write(&backup_path, &content) {
                                    println!("Failed to create backup file: {}", e);
                                    return HttpResponse::InternalServerError().json(ActionResponse {
                                        r#type: String::from("error"),
                                        media_type: String::from("text/plain"),
                                        data: format!("Failed to create backup: {}", e),
                                    });
                                }
                                
                                match fs::write(&req.path, new_content) {
                                    Ok(_) => HttpResponse::Ok().json(ActionResponse {
                                        r#type: String::from("success"),
                                        media_type: String::from("text/plain"),
                                        data: String::from("String replacement completed successfully"),
                                    }),
                                    Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                        r#type: String::from("error"),
                                        media_type: String::from("text/plain"),
                                        data: format!("Failed to write file: {}", e),
                                    })
                                }
                            },
                            Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                r#type: String::from("error"),
                                media_type: String::from("text/plain"),
                                data: format!("Failed to read file: {}", e),
                            })
                        }
                    } else {
                        HttpResponse::BadRequest().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: String::from("old_str and new_str are required for str_replace action"),
                        })
                    }
                },
                EditCommand::Insert => {
                    println!("Processing insert action");
                    if let (Some(text), Some(line_num)) = (&req.file_text, &req.insert_line) {
                        match fs::read_to_string(&req.path) {
                            Ok(content) => {
                                let mut lines: Vec<&str> = content.lines().collect();
                                let line_idx = *line_num as usize;
                                
                                // 创备份文件
                                let backup_path = format!("{}.bak", req.path);
                                if let Err(e) = fs::write(&backup_path, &content) {
                                    println!("Failed to create backup file: {}", e);
                                    return HttpResponse::InternalServerError().json(ActionResponse {
                                        r#type: String::from("error"),
                                        media_type: String::from("text/plain"),
                                        data: format!("Failed to create backup: {}", e),
                                    });
                                }

                                if line_idx <= lines.len() {
                                    lines.insert(line_idx, text);
                                    let new_content = lines.join("\n");
                                    match fs::write(&req.path, new_content) {
                                        Ok(_) => HttpResponse::Ok().json(ActionResponse {
                                            r#type: String::from("success"),
                                            media_type: String::from("text/plain"),
                                            data: String::from("Text inserted successfully"),
                                        }),
                                        Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                            r#type: String::from("error"),
                                            media_type: String::from("text/plain"),
                                            data: format!("Failed to write file: {}", e),
                                        })
                                    }
                                } else {
                                    HttpResponse::BadRequest().json(ActionResponse {
                                        r#type: String::from("error"),
                                        media_type: String::from("text/plain"),
                                        data: format!("Line number {} is out of range", line_num),
                                    })
                                }
                            },
                            Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                r#type: String::from("error"),
                                media_type: String::from("text/plain"),
                                data: format!("Failed to read file: {}", e),
                            })
                        }
                    } else {
                        HttpResponse::BadRequest().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: String::from("file_text and insert_line are required for insert action"),
                        })
                    }
                },
                EditCommand::UndoEdit => {
                    println!("Processing undo edit action");
                    let backup_path = format!("{}.bak", req.path);
                    if fs::metadata(&backup_path).is_ok() {
                        match fs::rename(&backup_path, &req.path) {
                            Ok(_) => HttpResponse::Ok().json(ActionResponse {
                                r#type: String::from("success"),
                                media_type: String::from("text/plain"),
                                data: String::from("Edit undone successfully"),
                            }),
                            Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                                r#type: String::from("error"),
                                media_type: String::from("text/plain"),
                                data: format!("Failed to restore backup: {}", e),
                            })
                        }
                    } else {
                        HttpResponse::BadRequest().json(ActionResponse {
                            r#type: String::from("error"),
                            media_type: String::from("text/plain"),
                            data: String::from("No backup file found to undo"),
                        })
                    }
                }
            }
        },
        None => {
            println!("Invalid command received: {}", req.command);
            HttpResponse::BadRequest().json(ActionResponse {
                r#type: String::from("error"),
                media_type: String::from("text/plain"),
                data: String::from("Unsupported edit command"),
            })
        }
    };
    
    println!("Edit command completed in {:?}", start.elapsed());
    response
}

#[get("/health")]
async fn health_check() -> impl Responder {
    log::info!("Health check called");
    HttpResponse::Ok()
        .json(ActionResponse {
            r#type: String::from("success"),
            media_type: String::from("text/plain"),
            data: String::from("Service is running")
        })
}

#[post("/computer")]
async fn computer_endpoint(req: web::Json<ActionRequest>) -> impl Responder {
    log::info!("Computer action received: {:?}", req);
    handle_computer_action(req).await
}

#[post("/edit")]
async fn edit_endpoint(req: web::Json<EditRequest>) -> impl Responder {
    log::info!("Edit command received: {:?}", req);
    handle_edit_action(req).await
}

#[derive(Debug, Deserialize)]
pub struct BashRequest {
    pub command: Option<String>,
    pub restart: Option<bool>,
}

#[derive(Debug)]
struct BashSession {
    process: tokio::process::Child,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    timed_out: bool,
}

impl BashSession {
    const TIMEOUT: Duration = Duration::from_secs(30);
    const OUTPUT_DELAY: Duration = Duration::from_millis(50);
    
    async fn new() -> Result<Self, String> {
        let mut child = tokio::process::Command::new("/bin/bash")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn bash: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

        Ok(BashSession {
            process: child,
            stdin,
            stdout,
            stderr,
            timed_out: false,
        })
    }

    async fn execute(&mut self, command: &str) -> Result<(String, String), String> {
        if self.timed_out {
            return Err("Session timed out".to_string());
        }

        let cmd = format!("{}\necho '---END---'\n", command);
        log::info!("Executing command: {}", cmd);
        
        self.stdin
            .write_all(cmd.as_bytes())
            .await
            .map_err(|e| {
                log::error!("Failed to write command: {}", e);
                format!("Failed to write command: {}", e)
            })?;
        
        self.stdin
            .flush()
            .await
            .map_err(|e| {
                log::error!("Failed to flush stdin: {}", e);
                format!("Failed to flush stdin: {}", e)
            })?;

        let mut output = String::new();
        let mut buffer = [0u8; 1024];

        // 使用timeout包装读取操作
        match timeout(Self::TIMEOUT, async {
            loop {
                match self.stdout.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        let chunk = String::from_utf8_lossy(&buffer[..n]);
                        output.push_str(&chunk);
                        log::debug!("Read chunk: {}", chunk);
                        
                        // 检查是否遇到结束标记
                        if output.contains("---END---") {
                            log::info!("Found end marker");
                            break;
                        }
                    }
                    Ok(_) => {
                        log::info!("Reached EOF");
                        break;
                    }
                    Err(e) => {
                        log::error!("Failed to read stdout: {}", e);
                        return Err(format!("Failed to read stdout: {}", e))
                    },
                }
            }
            Ok::<_, String>(())
        })
        .await
        {
            Ok(Ok(_)) => {
                // 移除结束标记
                if let Some(pos) = output.find("---END---") {
                    output.truncate(pos);
                }
                output = output.trim().to_string();
                log::info!("Command completed successfully with output: {}", output);
                
                Ok((output, String::new()))
            }
            Ok(Err(e)) => {
                log::error!("Command execution error: {}", e);
                Err(e)
            },
            Err(_) => {
                log::error!("Command execution timed out");
                self.timed_out = true;
                Err("Command execution timed out".to_string())
            }
        }
    }

    async fn stop(&mut self) -> Result<(), String> {
        // 发送退出命令
        self.stdin
            .write_all(b"exit\n")
            .await
            .map_err(|e| format!("Failed to send exit command: {}", e))?;
        
        // 等待进程结束
        match timeout(Duration::from_secs(5), self.process.wait()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                log::warn!("Process didn't exit cleanly: {}", e);
                // 强制结束进程
                self.process.kill().await.map_err(|e| format!("Failed to kill process: {}", e))?;
                Ok(())
            },
            Err(_) => {
                // 超时后强制结束进程
                self.process.kill().await.map_err(|e| format!("Failed to kill process: {}", e))?;
                Ok(())
            }
        }
    }
}


#[post("/bash")]
async fn bash_endpoint(req: web::Json<BashRequest>) -> impl Responder {
    log::info!("Received bash request: {:?}", req);
    
    let session_mutex = BASH_SESSION.get_or_init(|| Mutex::new(None));
    let mut session_guard = session_mutex.lock().unwrap();
    
    // 处理restart请求
    if req.restart.unwrap_or(false) {
        // 如果存在旧session，先停止它
        if let Some(mut session) = session_guard.take() {
            if let Err(e) = session.stop().await {
                log::warn!("Failed to stop bash session: {}", e);
            }
        }
        
        // 创建新session
        match BashSession::new().await {
            Ok(new_session) => {
                *session_guard = Some(new_session);
                return HttpResponse::Ok().json(ActionResponse {
                    r#type: String::from("success"),
                    media_type: String::from("text/plain"),
                    data: String::from("Bash session has been restarted"),
                });
            },
            Err(e) => return HttpResponse::InternalServerError().json(ActionResponse {
                r#type: String::from("error"),
                media_type: String::from("text/plain"),
                data: format!("Failed to restart bash session: {}", e),
            })
        }
    }

    // 处理命令请求
    if let Some(command) = &req.command {
        // 确保session存在
        if session_guard.is_none() {
            match BashSession::new().await {
                Ok(new_session) => {
                    *session_guard = Some(new_session);
                },
                Err(e) => return HttpResponse::InternalServerError().json(ActionResponse {
                    r#type: String::from("error"),
                    media_type: String::from("text/plain"),
                    data: format!("Failed to create bash session: {}", e),
                })
            }
        }

        // 执行命令
        if let Some(session) = session_guard.as_mut() {
            match session.execute(command).await {
                Ok((stdout, stderr)) => {
                    let response = if stderr.is_empty() {
                        stdout
                    } else {
                        format!("stdout:\n{}\nstderr:\n{}", stdout, stderr)
                    };
                    
                    HttpResponse::Ok().json(ActionResponse {
                        r#type: String::from("success"),
                        media_type: String::from("text/plain"),
                        data: response,
                    })
                }
                Err(e) => HttpResponse::InternalServerError().json(ActionResponse {
                    r#type: String::from("error"),
                    media_type: String::from("text/plain"),
                    data: format!("Command execution failed: {}", e),
                })
            }
        } else {
            HttpResponse::InternalServerError().json(ActionResponse {
                r#type: String::from("error"),
                media_type: String::from("text/plain"),
                data: String::from("No active bash session"),
            })
        }
    } else {
        HttpResponse::BadRequest().json(ActionResponse {
            r#type: String::from("error"),
            media_type: String::from("text/plain"),
            data: String::from("Invalid request: command is required when not restarting"),
        })
    }
}

pub fn run(listener: std::net::TcpListener) -> std::io::Result<actix_web::dev::Server> {
    // 在程序启动时立即打印
    eprintln!("=== Server starting ===");  // 使用 eprintln! 确保输出到标准错误
    
    let server = HttpServer::new(move || {
        eprintln!("=== Creating new worker ===");  // 添加工作进程创建日志
        App::new()
            .wrap(Logger::default())
            .wrap(actix_web::middleware::NormalizePath::trim())
            .service(health_check)
            .service(computer_endpoint)
            .service(edit_endpoint)
            .service(bash_endpoint)
            .app_data(web::JsonConfig::default().limit(4096 * 1024))
    })
    .keep_alive(actix_web::http::KeepAlive::Timeout(std::time::Duration::from_secs(60)))
    .workers(4)
    .listen(listener)?
    .run();

    eprintln!("=== Server started ===");  // 添加服务器启动完成日志
    Ok(server)
} 