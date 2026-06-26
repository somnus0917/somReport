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

    pub fn check_and_perform_cache_cleanup(&self, auto_cleanup_cache_days: u32) -> Result<(), String> {
        if auto_cleanup_cache_days == 0 {
            return Ok(());
        }
        let last_cleanup: Option<String> = self.conn()
            .query_row(
                "SELECT value FROM settings WHERE key = 'last_cache_cleanup_at'",
                [],
                |row| row.get(0),
            )
            .ok();

        let should_cleanup = match last_cleanup {
            Some(date_str) => {
                if let Ok(last_date) = chrono::DateTime::parse_from_rfc3339(&date_str) {
                    let days_since = (Utc::now() - last_date.with_timezone(&Utc)).num_days();
                    days_since >= i64::from(auto_cleanup_cache_days)
                } else {
                    true
                }
            }
            None => true,
        };

        if should_cleanup {
            log::info!("Performing periodic automatic cache cleanup...");
            crate::platform::paths::clear_cache().map_err(|error| error.to_string())?;
            let now_str = Utc::now().to_rfc3339();
            self.conn().execute(
                "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES ('last_cache_cleanup_at', ?1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                rusqlite::params![now_str],
            ).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn checkpoint_wal(&self) -> Result<(), String> {
        self.conn()
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|error| error.to_string())
    }

    pub fn backup_database(&self) -> Result<(), String> {
        let db_path = crate::platform::paths::db_path();
        let mut backup_path = db_path.clone();
        backup_path.set_extension("db.bak");
        std::fs::copy(&db_path, &backup_path)
            .map_err(|error| format!("Failed to copy database file: {error}"))?;
        Ok(())
    }
}
