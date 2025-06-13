use anyhow::Result;
use axum::extract::Query;
use moly_kit::utils::asynchronous::spawn;
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::crypto::encrypt_json;

/// Server handle that can be used to stop the server
#[derive(Debug)]
pub struct ServerHandle {
    pub addr: std::net::SocketAddr,
    pub pin: String,
    shutdown_tx: oneshot::Sender<()>,
}

impl ServerHandle {
    /// Stop the server gracefully
    pub fn stop(self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Start a simple HTTP server that serves encrypted JSON file and return a handle to stop it
pub async fn start_server(json_file: String, port: Option<u16>) -> Result<ServerHandle> {
    use axum::{routing::get, Router};
    use tower_http::cors::CorsLayer;

    let port = port.unwrap_or(0); // 0 = any available port
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    let addr = listener.local_addr()?;

    // Generate a random 4 digit PIN
    let pin = format!("{:04}", rand::rng().random_range(0..=9999));

    // Pre-encrypt the JSON data with the PIN
    let encrypted_json = encrypt_json(&json_file, &pin)
        .map_err(|e| anyhow::anyhow!("Failed to encrypt preferences data: {}", e))?;

    let app = Router::new()
        .route(
            "/preferences.json",
            get({
                let token = pin.clone();
                let encrypted_data = encrypted_json.clone();
                move |Query(query): Query<HashMap<String, String>>| async move {
                    if query.get("token") == Some(&token) {
                        Ok(encrypted_data)
                    } else {
                        ::log::warn!("Invalid token provided for preferences access");
                        Err(axum::http::StatusCode::UNAUTHORIZED)
                    }
                }
            }),
        )
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive());

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

        if let Err(e) = server.await {
            log::error!("Server error: {}", e);
        }
    });

    Ok(ServerHandle {
        addr,
        shutdown_tx,
        pin,
    })
}
