//! Session Manager Module
//! Track and manage connected session

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::database::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconData {
    pub session_id: String,
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub pid: u32,
    pub process: String,
    pub arch: String,
    pub integrity: String,
    pub timestamp: i64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
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
    pub metadata: serde_json::Value,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Dead,
    Stale
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    pub db: Database,
}

impl SessionManager {
    pub fn new(db: Database) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            db
        }
    }

    pub async fn register(&self, data: &BeaconData) -> Result<Session, anyhow::Error> {
        let mut sessions = self.sessions.write().await;

        let now = chrono::Utc::now().timestamp();

        let session = if let Some(existing) = sessions.get_mut(&data.session_id) {
            existing.last_seen = now;
            existing.checkins += 1;
            existing.status = SessionStatus::Active;
            existing.clone()
        } else {
            let session = Session {
                id: data.session_id.clone(),
                hostname: data.hostname.clone(),
                username: data.username.clone(),
                os: data.os.clone(),
                arch: data.arch.clone(),
                pid: data.pid,
                process: data.process.clone(),
                integrity: data.integrity.clone(),
                first_seen: now,
                last_seen: now,
                checkins: 1,
                metadata: data.metadata.clone(),
                status: SessionStatus::Active,
            };

            // Save to database
            self.db.save(&session)?; // This function will return Result, do not forget to use "?"

            sessions.insert(session.id.clone(), session.clone());
            session
        };

        Ok(session)
    }

    pub async fn get(&self, id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }
    
    pub async fn list(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn remove(&self, id: &str) -> Result<(), anyhow::Error> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        // self.db.remove_session(id)?; // function is not implemented yet
        Ok(())
        }
}