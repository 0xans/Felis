//! Command Queue Module
//! Queue and manage commands for sessions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    Shell, Download, Upload, Screenshot, Ps, Kill,
    BofLoad, Sysinfo, Sleep, Exit, Ls, Cd,
    Pwd, Rm, Cp, Mv, RegRead, RegWrite, Persist, Inject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub session_id: String,
    pub command_type: CommandType,
    pub args: Vec<String>,
    pub timeout: Option<u64>,
    pub status: CommandStatus,
    pub created_at: i64,
    pub result: Option<CommandResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandStatus {
    Queued,
    Sent,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub id: String,
    pub output: String,
    pub success: bool,
    pub duration_ms: u64,
    pub data: Option<String>,
}

#[derive(Clone)]
pub struct CommandQueue {
    queues: Arc<RwLock<HashMap<String, Vec<Command>>>>,
    history: Arc<RwLock<HashMap<String, Command>>>
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    /* Queue a new command */
    pub async fn queue(&self, session_id: &str, command_type: CommandType, args: Vec<String>, timeout: Option<u64>) -> Result<Command, anyhow::Error> {
        let mut queues = self.queues.write().await;
        let mut history = self.history.write().await;

        let command = Command {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            command_type,
            args,
            timeout,
            status: CommandStatus::Queued,
            created_at: chrono::Utc::now().timestamp(),
            result: None,
        };

        queues.entry(session_id.to_string()).or_insert_with(Vec::new).push(command.clone());
        history.insert(command.id.clone(), command.clone());  
            
        log::info!("Queued command {} for session {}: {:?}", command.id, session_id, command.command_type);

        Ok(command)
    }

    /* Get pending command for a session */
    // This will return all pending commands for a session and mark them as sent
    pub async fn pending(&self, sessions_id: &str) -> Vec<Command> {
        let mut queues = self.queues.write().await;

        if let Some(queue) = queues.get_mut(sessions_id) {
            let pending: Vec<Command> = queue.iter().filter(|c| c.status == CommandStatus::Queued).cloned().collect();                        
            
            for cmd in queue.iter_mut() {
                if cmd.status == CommandStatus::Queued {
                    cmd.status = CommandStatus::Sent
                }
            }

            return pending;
        }
        Vec::new()
    }

    /* Update command result */
    pub async fn result(&self, command_id: &str, result: CommandResult) -> Result<(), anyhow::Error> {
        let mut queues = self.queues.write().await;
        let mut history = self.history.write().await;

        for (_, queue) in queues.iter_mut() {
            if let Some(cmd) = queue.iter_mut().find(|c| c.id == command_id) {
                cmd.status = if result.success {
                    CommandStatus::Completed
                } else {
                    CommandStatus::Failed
                };
                cmd.result = Some(result.clone());
                break;
            }
        }
        
        if let Some(cmd) = history.get_mut(command_id) {
            cmd.status = if result.success {
                CommandStatus::Completed
            } else {
                CommandStatus::Failed
            };
            cmd.result = Some(result);
        }
        Ok(())
    }

    /* Get command by id */
    pub async fn get(&self, command_id: &str) -> Option<Command> {
        let history = self.history.write().await;
        history.get(command_id).cloned()
    }

    /* List all command  */
    pub async fn all(&self) -> Vec<Command> {
        let history = self.history.write().await;
        history.values().cloned().collect()
    }

    /* List command for a session */
    pub async fn list_for_session(&self, session_id: &str) -> Vec<Command> {
        let history = self.history.write().await;
        history.values().filter(|c| c.session_id == session_id).cloned().collect()
    }

    /* Clear completed and faild commands for a session */
    pub async fn clear_completed(&self, session_id: &str) {
        let mut queues = self.queues.write().await;
        if let Some(queue) = queues.get_mut(session_id) {
            queue.retain(|c| {
                c.status != CommandStatus::Completed && c.status != CommandStatus::Failed
            });
        }
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}