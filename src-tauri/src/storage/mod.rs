pub mod activity_repo;
pub mod job_repo;
pub mod migrations;
pub mod report_repo;
pub mod settings_repo;
pub mod usage_repo;

use chrono::Utc;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(db_path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;",
        )?;
        migrations::run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn new_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        migrations::run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("database lock poisoned")
    }

    pub fn purge_records_older_than(&self, retention_days: u32) -> Result<(), String> {
        if retention_days == 0 {
            return Ok(());
        }
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(retention_days));
        let cutoff = cutoff.to_rfc3339();
        let conn = self.conn();
        conn.execute("DELETE FROM activities WHERE started_at < ?1", [&cutoff])
            .map_err(|error| error.to_string())?;
        conn.execute("DELETE FROM api_usage WHERE occurred_at < ?1", [&cutoff])
            .map_err(|error| error.to_string())?;
        conn.execute("DELETE FROM reports WHERE period_end < ?1", [&cutoff])
            .map_err(|error| error.to_string())?;
        conn.execute(
            "DELETE FROM analysis_jobs WHERE captured_at < ?1 AND id NOT IN (SELECT DISTINCT job_id FROM activities)",
            [&cutoff],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn checkpoint_wal(&self) -> Result<(), String> {
        self.conn()
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|error| error.to_string())
    }
}
