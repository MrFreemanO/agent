use env_logger;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging system
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    log::info!("Starting server...");
    
    // Bind to all interfaces
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");
    log::info!("Listening on http://0.0.0.0:8080");
    
    let server = api_server::run(listener)?;
    
    // Wait for server to run
    server.await
} 