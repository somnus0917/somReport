use chrono::{DateTime, Utc};
use rusqlite::params;

use super::Database;
use crate::domain::{AnalysisJob, JobStatus};

fn row_to_job(row: &rusqlite::Row) -> rusqlite::Result<AnalysisJob> {
    let captured_at_str: String = row.get("captured_at")?;
    let status_str: String = row.get("status")?;
    let created_at_str: String = row.get("created_at")?;
    let finished_at_str: Option<String> = row.get("finished_at")?;

    let captured_at = captured_at_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let created_at = created_at_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let finished_at = match finished_at_str {
        Some(s) => Some(
            s.parse::<DateTime<Utc>>()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
        ),
        None => None,
    };

    Ok(AnalysisJob {
        id: row.get("id")?,
        captured_at,
        status: JobStatus::from_str(&status_str),
        attempts: row.get("attempts")?,
        last_error: row.get("last_error")?,
        image_hash: row.get("image_hash")?,
        provider: row.get("provider")?,
        model: row.get("model")?,
        created_at,
        finished_at,
    })
}

impl Database {
    pub fn insert_job(&self, job: &AnalysisJob) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO analysis_jobs (id, captured_at, status, attempts, last_error, image_hash, provider, model, created_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                job.id,
                job.captured_at.to_rfc3339(),
                job.status.as_str(),
                job.attempts,
                job.last_error,
                job.image_hash,
                job.provider,
                job.model,
                job.created_at.to_rfc3339(),
                job.finished_at.map(|d| d.to_rfc3339()),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn claim_next_pending_job(&self) -> Result<Option<AnalysisJob>, String> {
        let conn = self.conn();

        let job = {
            let mut stmt = conn
                .prepare(
                    "SELECT id, captured_at, status, attempts, last_error, image_hash, provider, model, created_at, finished_at
                     FROM analysis_jobs
                     WHERE status IN ('pending', 'processing') AND attempts < 3
                     ORDER BY created_at
                     LIMIT 1",
                )
                .map_err(|e| e.to_string())?;

            let mut rows = stmt.query_map([], row_to_job).map_err(|e| e.to_string())?;

            match rows.next() {
                Some(row) => Some(row.map_err(|e| e.to_string())?),
                None => None,
            }
        };

        let Some(mut job) = job else {
            return Ok(None);
        };

        conn.execute(
            "UPDATE analysis_jobs SET status = 'processing', attempts = attempts + 1 WHERE id = ?1",
            params![job.id],
        )
        .map_err(|e| e.to_string())?;

        job.status = JobStatus::Processing;
        job.attempts += 1;

        Ok(Some(job))
    }

    pub fn complete_job(&self, id: &str, provider: &str, model: &str) -> Result<(), String> {
        let conn = self.conn();
        let now = Utc::now().to_rfc3339();
        let rows = conn.execute(
            "UPDATE analysis_jobs SET status = 'completed', provider = ?2, model = ?3, finished_at = ?4 WHERE id = ?1",
            params![id, provider, model, now],
        ).map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err(format!("job {} not found", id));
        }
        Ok(())
    }

    pub fn fail_job(&self, id: &str, error: &str) -> Result<(), String> {
        let conn = self.conn();
        let now = Utc::now().to_rfc3339();
        let rows = conn.execute(
            "UPDATE analysis_jobs SET status = 'failed', last_error = ?2, finished_at = ?3 WHERE id = ?1",
            params![id, error, now],
        ).map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err(format!("job {} not found", id));
        }
        Ok(())
    }

    pub fn retry_job(&self, id: &str) -> Result<(), String> {
        let conn = self.conn();
        let rows = conn.execute(
            "UPDATE analysis_jobs SET status = 'pending', last_error = NULL WHERE id = ?1 AND status = 'failed'",
            params![id],
        ).map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err(format!("job {} not found or not in failed status", id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("failed to create in-memory db")
    }

    fn make_job() -> AnalysisJob {
        AnalysisJob {
            id: Uuid::new_v4().to_string(),
            captured_at: Utc::now(),
            status: JobStatus::Pending,
            attempts: 0,
            last_error: None,
            image_hash: Some("abc123".to_string()),
            provider: None,
            model: None,
            created_at: Utc::now(),
            finished_at: None,
        }
    }

    #[test]
    fn test_insert_and_claim() {
        let db = setup_db();
        let job = make_job();
        db.insert_job(&job).unwrap();

        let claimed = db.claim_next_pending_job().unwrap().unwrap();
        assert_eq!(claimed.id, job.id);
        assert_eq!(claimed.status, JobStatus::Processing);
        assert_eq!(claimed.attempts, 1);
    }

    #[test]
    fn test_claim_sets_processing() {
        let db = setup_db();
        let job = make_job();
        db.insert_job(&job).unwrap();

        let claimed = db.claim_next_pending_job().unwrap().unwrap();
        assert_eq!(claimed.status, JobStatus::Processing);
        assert_eq!(claimed.attempts, 1);

        let claimed2 = db.claim_next_pending_job().unwrap().unwrap();
        assert_eq!(claimed2.id, job.id);
        assert_eq!(claimed2.attempts, 2);

        let claimed3 = db.claim_next_pending_job().unwrap().unwrap();
        assert_eq!(claimed3.attempts, 3);

        let none = db.claim_next_pending_job().unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_complete_job() {
        let db = setup_db();
        let job = make_job();
        db.insert_job(&job).unwrap();

        db.complete_job(&job.id, "openai", "gpt-4o").unwrap();

        let conn = db.conn();
        let (status, provider, model, finished_at): (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = conn
            .query_row(
                "SELECT status, provider, model, finished_at FROM analysis_jobs WHERE id = ?1",
                params![job.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert_eq!(status, "completed");
        assert_eq!(provider.as_deref(), Some("openai"));
        assert_eq!(model.as_deref(), Some("gpt-4o"));
        assert!(finished_at.is_some());
    }

    #[test]
    fn test_fail_job() {
        let db = setup_db();
        let job = make_job();
        db.insert_job(&job).unwrap();

        db.fail_job(&job.id, "rate limited").unwrap();

        let conn = db.conn();
        let (status, last_error, finished_at): (String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT status, last_error, finished_at FROM analysis_jobs WHERE id = ?1",
                params![job.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(status, "failed");
        assert_eq!(last_error.as_deref(), Some("rate limited"));
        assert!(finished_at.is_some());
    }
}
