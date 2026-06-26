use chrono::{DateTime, Utc};
use rusqlite::params;

use super::Database;
use crate::domain::{PeriodType, Report};

fn row_to_report(row: &rusqlite::Row) -> rusqlite::Result<Report> {
    let period_type_str: String = row.get("period_type")?;
    let period_start_str: String = row.get("period_start")?;
    let period_end_str: String = row.get("period_end")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let period_start = period_start_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let period_end = period_end_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let created_at = created_at_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let updated_at = updated_at_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

    Ok(Report {
        id: row.get("id")?,
        period_type: PeriodType::from_str(&period_type_str),
        period_start,
        period_end,
        template_id: row.get("template_id")?,
        title: row.get("title")?,
        content_markdown: row.get("content_markdown")?,
        model: row.get("model")?,
        prompt_version: row.get("prompt_version")?,
        created_at,
        updated_at,
    })
}

impl Database {
    pub fn insert_report(&self, report: &Report) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO reports (id, period_type, period_start, period_end, template_id, title, content_markdown, model, prompt_version, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                report.id,
                report.period_type.as_str(),
                report.period_start.to_rfc3339(),
                report.period_end.to_rfc3339(),
                report.template_id,
                report.title,
                report.content_markdown,
                report.model,
                report.prompt_version,
                report.created_at.to_rfc3339(),
                report.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_reports(&self, period_type: Option<PeriodType>) -> Result<Vec<Report>, String> {
        let conn = self.conn();

        let (sql, filter_value): (&str, Option<String>) = match &period_type {
            Some(pt) => (
                "SELECT id, period_type, period_start, period_end, template_id, title, content_markdown, model, prompt_version, created_at, updated_at
                 FROM reports WHERE period_type = ?1 ORDER BY period_start DESC",
                Some(pt.as_str().to_string()),
            ),
            None => (
                "SELECT id, period_type, period_start, period_end, template_id, title, content_markdown, model, prompt_version, created_at, updated_at
                 FROM reports ORDER BY period_start DESC",
                None,
            ),
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let reports = if let Some(ref val) = filter_value {
            stmt.query_map(params![val], row_to_report)
        } else {
            stmt.query_map([], row_to_report)
        }
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(reports)
    }

    pub fn delete_report(&self, id: &str) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM reports WHERE id = ?1",
            params![id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use uuid::Uuid;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("failed to create in-memory db")
    }

    fn make_report(period_type: PeriodType, title: &str) -> Report {
        Report {
            id: Uuid::new_v4().to_string(),
            period_type,
            period_start: Utc.with_ymd_and_hms(2025, 6, 15, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2025, 6, 15, 23, 59, 59).unwrap(),
            template_id: None,
            title: title.to_string(),
            content_markdown: "# Report\n\nContent here".to_string(),
            model: Some("gpt-4o".to_string()),
            prompt_version: Some("v1".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_insert_and_get_reports() {
        let db = setup_db();

        let r1 = make_report(PeriodType::Daily, "Daily Report");
        let r2 = make_report(PeriodType::Weekly, "Weekly Report");

        db.insert_report(&r1).unwrap();
        db.insert_report(&r2).unwrap();

        let all = db.get_reports(None).unwrap();
        assert_eq!(all.len(), 2);

        let dailies = db.get_reports(Some(PeriodType::Daily)).unwrap();
        assert_eq!(dailies.len(), 1);
        assert_eq!(dailies[0].title, "Daily Report");

        let weeklies = db.get_reports(Some(PeriodType::Weekly)).unwrap();
        assert_eq!(weeklies.len(), 1);
        assert_eq!(weeklies[0].title, "Weekly Report");

        let customs = db.get_reports(Some(PeriodType::Custom)).unwrap();
        assert_eq!(customs.len(), 0);
    }
}
