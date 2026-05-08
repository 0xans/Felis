use axum::{body::Bytes, extract::State, http::{HeaderMap, StatusCode}};
use std::sync::Arc;
use crate::{server::ServerState, session::BeaconData};

pub async fn beacon(State(state): State<Arc<ServerState>>, _headers: HeaderMap, body: Bytes,) -> Result<Bytes, StatusCode> {

    let decrypted = state.crypto.decrypt(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    let data: BeaconData = serde_json::from_slice(&decrypted).map_err(|_| StatusCode::BAD_REQUEST)?;

    log::debug!("Beacon donnection received from session {} ({}@{})", data.session_id, data.username, data.hostname);

    // Register/update session
    let _session: crate::session::Session = state.sessions.register(&data).await.map_err(|e| {
        log::error!("Faild to register session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Bytes::from("OK"))
}