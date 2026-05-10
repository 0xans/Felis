//! Database Module
//! SQLite presistence for sessions and commands

use anyhow::Result;
use rusqlite::params;
use std::sync::{Arc, Mutex};

use crate::command::Command;
use crate::session::{Session, SessionStatus};

pub struct Database {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

impl Database {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = rusqlite::Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    hostname TEXT NOT NULL,
                    username TEXT NOT NULL,
                    os TEXT NOT NULL,
                    arch TEXT NOT NULL,
                    pid INTEGER NOT NULL,
                    process TEXT NOT NULL,
                    integrity TEXT NOT NULL,
                    first_seen INTEGER NOT NULL,
                    last_seen INTEGER NOT NULL,
                    checkins INTEGER NOT NULL,
                    metadata TEXT,
                    status TEXT NOT NULL
                );
 
                CREATE TABLE IF NOT EXISTS commands (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    command_type TEXT NOT NULL,
                    args TEXT NOT NULL,
                    timeout INTEGER,
                    status TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    result TEXT,
                    FOREIGN KEY(session_id) REFERENCES sessions(id)
                );

                CREATE TABLE IF NOT EXISTS logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT,
                    level TEXT NOT NULL,
                    message TEXT NOT NULL,
                    timestamp INTEGER NOT NULL
                );
            "#,
        )?;
        Ok(())
    }

    pub fn save_session(&self, session: &Session) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
                INSERT OR REPLACE INTO sessions 
                (id, hostname, username, os, arch, pid, 
                 process, integrity, first_seen, last_seen, checkins, metadata, status)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                session.id,
                session.hostname,
                session.username,
                session.os,
                session.arch,
                session.pid,
                session.process,
                session.integrity,
                session.first_seen,
                session.last_seen,
                session.checkins,
                serde_json::to_string(&session.metadata)?,
                serde_json::to_string(&session.status)?,
            ],
        )?;
        Ok(())
    }

    pub fn load_sessions(&self) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT id, hostname, username, os, arch, pid, process, integrity, first_seen, last_seen, checkins, metadata, status FROM sessions")?;

        let sessions = stmt
            .query_map([], |row: &rusqlite::Row| {
                Ok(Session {
                    id: row.get(0)?,
                    hostname: row.get(1)?,
                    username: row.get(2)?,
                    os: row.get(3)?,
                    arch: row.get(4)?,
                    pid: row.get(5)?,
                    process: row.get(6)?,
                    integrity: row.get(7)?,
                    first_seen: row.get(8)?,
                    last_seen: row.get(9)?,
                    checkins: row.get(10)?,
                    metadata: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
                    status: serde_json::from_str(&row.get::<_, String>(12)?)
                        .unwrap_or(SessionStatus::Active),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    }

    pub fn remove_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn save_command(&self, command: &Command) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
                INSERT OR REPLACE INTO commands
                (id, session_id, command_type, args, timeout, status, created_at, result)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                command.id,
                command.session_id,
                serde_json::to_string(&command.command_type)?,
                serde_json::to_string(&command.args)?,
                command.timeout,
                serde_json::to_string(&command.status)?,
                command.created_at,
                serde_json::to_string(&command.result)?,
            ],
        )?;
        Ok(())
    }

    pub fn load_commands(&self) -> Result<Vec<Command>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
                SELECT id, session_id, command_type, args, timeout, status, created_at, result
                FROM commands
            "#,
        )?;

        let commands = stmt
            .query_map([], |row: &rusqlite::Row| {
                let command_type: String = row.get(2)?;
                let args: String = row.get(3)?;
                let status: String = row.get(5)?;
                let result: String = row.get(7)?;

                Ok(Command {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    command_type: serde_json::from_str(&command_type)
                        .unwrap_or(crate::command::CommandType::Sysinfo),
                    args: serde_json::from_str(&args).unwrap_or_default(),
                    timeout: row.get(4)?,
                    status: serde_json::from_str(&status)
                        .unwrap_or(crate::command::CommandStatus::Failed),
                    created_at: row.get(6)?,
                    result: serde_json::from_str(&result).unwrap_or(None),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(commands)
    }
}
