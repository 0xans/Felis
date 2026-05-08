//! Database Module
//! SQLite presistence for sessions and commands

use anyhow::Result;
use rusqlite::params;
use std::sync::{Arc, Mutex};

use crate::session::Session;

pub struct Database {
    conn: Arc<Mutex<rusqlite::Connection>>
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn)
        }
    }
}

impl Database {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = rusqlite::Connection::open(path)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
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
            "#,
        )?;
        Ok(())
    }

    pub fn save(&self, session: &Session) -> Result<()> { // This should return result "-> Result<()>"
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
            ]
        )?;
        Ok(())
    }
}