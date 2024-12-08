use actix_web::middleware::Logger;
use env_logger;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志系统
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    log::info!("Starting server...");
    
    // 明确绑定到所有接口
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");
    log::info!("Listening on http://0.0.0.0:8080");
    
    let server = api_server::run(listener)?;
    
    // 等待服务器运行
    server.await
} 