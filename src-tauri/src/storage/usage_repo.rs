use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub id: String,
    pub occurred_at: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub estimated_cost_cents: f64,
    pub job_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub estimated_cost_cents: f64,
}

impl Database {
    pub fn record_usage(&self, entry: &UsageEntry) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO api_usage (id, occurred_at, provider, model, input_tokens, output_tokens, estimated_cost_cents, job_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id,
                entry.occurred_at.to_rfc3339(),
                entry.provider,
                entry.model,
                entry.input_tokens,
                entry.output_tokens,
                entry.estimated_cost_cents,
                entry.job_id,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_daily_usage_cents(&self, date: NaiveDate) -> Result<f64, String> {
        Ok(self.get_daily_usage(date)?.estimated_cost_cents)
    }

    pub fn get_daily_usage(&self, date: NaiveDate) -> Result<DailyUsage, String> {
        let start = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = date
            .succ_opt()
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let start_utc = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
        let end_utc = DateTime::<Utc>::from_naive_utc_and_offset(end, Utc);

        let conn = self.conn();
        let usage = conn
            .query_row(
                "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(estimated_cost_cents), 0)
                 FROM api_usage
                 WHERE occurred_at >= ?1 AND occurred_at < ?2",
                params![start_utc.to_rfc3339(), end_utc.to_rfc3339()],
                |row| {
                    Ok(DailyUsage {
                        input_tokens: row.get(0)?,
                        output_tokens: row.get(1)?,
                        estimated_cost_cents: row.get(2)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?;

        Ok(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("failed to create in-memory db")
    }

    fn make_entry(date: NaiveDate, hour: u32, cost_cents: f64) -> UsageEntry {
        let dt =
            DateTime::<Utc>::from_naive_utc_and_offset(date.and_hms_opt(hour, 0, 0).unwrap(), Utc);
        UsageEntry {
            id: Uuid::new_v4().to_string(),
            occurred_at: dt,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            estimated_cost_cents: cost_cents,
            job_id: None,
        }
    }

    #[test]
    fn test_record_and_get_daily_usage() {
        let db = setup_db();

        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let e1 = make_entry(date, 9, 50.0);
        let e2 = make_entry(date, 14, 75.0);

        let other_date = NaiveDate::from_ymd_opt(2025, 6, 16).unwrap();
        let e3 = make_entry(other_date, 10, 100.0);

        db.record_usage(&e1).unwrap();
        db.record_usage(&e2).unwrap();
        db.record_usage(&e3).unwrap();

        let total = db.get_daily_usage_cents(date).unwrap();
        assert_eq!(total, 125.0);

        let other_total = db.get_daily_usage_cents(other_date).unwrap();
        assert_eq!(other_total, 100.0);

        let empty_date = NaiveDate::from_ymd_opt(2025, 6, 14).unwrap();
        let empty_total = db.get_daily_usage_cents(empty_date).unwrap();
        assert_eq!(empty_total, 0.0);
    }
}
