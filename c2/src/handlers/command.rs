//! Command Handler
//! Handle command managment API requests

use crate::server::ServerState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;
use crate::command::{CommandType, CommandStatus, Command};

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub id: String,
    pub session_id: String,
    pub command_type: String,
    pub args: Vec<String>,
    pub timeout: Option<u64>,
    pub status: String,
    pub created_at: i64,
    pub result: Option<CommandResultInfo> 
}

#[derive(Debug, Serialize)]
pub struct CommandResultInfo {
    pub output: String,
    pub success: bool, 
    pub duration_ms: u64,
    pub data: Option<String>, 
}

pub async fn list_commands(State(state): State<Arc<ServerState>>) -> Result<Json<Vec<CommandInfo>>, StatusCode> {
    let commands = state.commands.all().await;
    let info: Vec<CommandInfo> = commands.into_iter().map(|c| c.into()).collect();
    Ok(Json(info))
}

impl From<Command> for CommandInfo {
    fn from(cmd: Command) -> Self {
        Self {
            id: cmd.id,
            session_id: cmd.session_id,
            command_type: match cmd.command_type {
                CommandType::Shell => "shell",
                CommandType::Download => "download",
                CommandType::Upload => "upload",
                CommandType::Screenshot => "screenshot",
                CommandType::Ps => "ps",
                CommandType::Kill => "kill",
                CommandType::BofLoad => "bof_load",
                CommandType::Sysinfo => "sysinfo",
                CommandType::Sleep => "sleep",
                CommandType::Exit => "exit",
                CommandType::Ls => "ls",
                CommandType::Cd => "cd",
                CommandType::Pwd => "pwd",
                CommandType::Rm => "rm",
                CommandType::Cp => "cp",
                CommandType::Mv => "mv",
                CommandType::RegRead => "reg_read",
                CommandType::RegWrite => "reg_write",
                CommandType::Persist => "persist",
                CommandType::Inject => "inject",
            }.to_string(),
            args: cmd.args,
            timeout: cmd.timeout,
            status: match cmd.status {
                CommandStatus::Queued => "queued",
                CommandStatus::Sent => "sent",
                CommandStatus::Completed => "completed",
                CommandStatus::Failed => "failed",
                CommandStatus::Timeout => "timeout",
            }.to_string(),
            created_at: cmd.created_at,
            result: cmd.result.map(|r| CommandResultInfo { 
                output: r.output, 
                success: r.success, 
                duration_ms: r.duration_ms, 
                data: r.data 
            }),
        }
    }
}