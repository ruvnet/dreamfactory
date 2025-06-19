use anyhow::Result;
use axum::Router;
use clap::Parser;
use df_api::{create_api_router, init_tracing, ApiConfig};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::{
    compression::CompressionLayer,
    timeout::TimeoutLayer,
    limit::RequestBodyLimitLayer,
};
use tracing::info;

mod config;
use config::ServerConfig;

#[derive(Parser)]
#[command(name = "df-server")]
#[command(about = "DreamFactory REST API Server")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Development mode
    #[arg(long)]
    dev: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    init_tracing(&cli.log_level)?;

    info!("Starting DreamFactory API Server v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let server_config = ServerConfig::load(&cli.config)?;
    info!("Configuration loaded from: {}", cli.config);

    // Create API configuration
    let api_config = ApiConfig {
        host: cli.host.clone(),
        port: cli.port,
        enable_cors: server_config.cors.enabled,
        log_level: cli.log_level.clone(),
    };

    // Create the main application router
    let app = create_application_router(api_config, server_config).await?;

    // Create socket address
    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    info!("Server listening on http://{}", addr);

    // Create the TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Start the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Create the main application router with all middleware
async fn create_application_router(
    _api_config: ApiConfig,
    server_config: ServerConfig,
) -> Result<Router> {
    info!("Creating application router");

    // Create the API router
    let api_router = create_api_router()?;

    // Build the main application with middleware
    let app = Router::new()
        .merge(api_router)
        // Apply compression
        .layer(CompressionLayer::new())
        // Apply timeout
        .layer(TimeoutLayer::new(
            std::time::Duration::from_secs(server_config.server.timeout_seconds)
        ))
        // Apply request body size limit
        .layer(RequestBodyLimitLayer::new(
            server_config.server.max_request_size
        ));

    info!("Application router created successfully");
    Ok(app)
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        },
    }

    info!("Shutdown signal received, cleaning up...");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_server_config_creation() {
        let config = ServerConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
    }

    #[tokio::test]
    async fn test_application_router_creation() {
        let api_config = ApiConfig::default();
        let server_config = ServerConfig::default();

        let router = create_application_router(api_config, server_config).await;
        assert!(router.is_ok());
    }

    #[tokio::test]
    async fn test_cli_parsing() {
        let cli = Cli::parse_from(&["df-server", "--port", "3000", "--host", "0.0.0.0"]);
        assert_eq!(cli.port, 3000);
        assert_eq!(cli.host, "0.0.0.0");
    }

    #[tokio::test]
    async fn test_shutdown_signal_timeout() {
        // Test that shutdown signal doesn't hang indefinitely
        let shutdown_future = shutdown_signal();
        let result = timeout(Duration::from_millis(100), shutdown_future).await;
        assert!(result.is_err()); // Should timeout since no signal is sent
    }
}