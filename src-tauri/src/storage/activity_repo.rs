use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rusqlite::params;

use super::Database;
use crate::domain::{Activity, Category};

fn row_to_activity(row: &rusqlite::Row) -> rusqlite::Result<Activity> {
    let started_at_str: String = row.get("started_at")?;
    let ended_at_str: String = row.get("ended_at")?;
    let edited_at_str: Option<String> = row.get("edited_at")?;
    let deleted_at_str: Option<String> = row.get("deleted_at")?;
    let category_str: String = row.get("category")?;
    let is_work_related_int: i32 = row.get("is_work_related")?;

    let started_at = started_at_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let ended_at = ended_at_str
        .parse::<DateTime<Utc>>()
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let edited_at = match edited_at_str {
        Some(s) => Some(
            s.parse::<DateTime<Utc>>()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
        ),
        None => None,
    };
    let deleted_at = match deleted_at_str {
        Some(s) => Some(
            s.parse::<DateTime<Utc>>()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
        ),
        None => None,
    };

    Ok(Activity {
        id: row.get("id")?,
        job_id: row.get("job_id")?,
        started_at,
        ended_at,
        category: Category::from_str(&category_str),
        summary: row.get("summary")?,
        detail: row.get("detail")?,
        confidence: row.get("confidence")?,
        is_work_related: is_work_related_int != 0,
        source: row.get("source")?,
        edited_at,
        deleted_at,
    })
}

impl Database {
    pub fn insert_activity(&self, activity: &Activity) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO activities (id, job_id, started_at, ended_at, category, summary, detail, confidence, is_work_related, source, edited_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                activity.id,
                activity.job_id,
                activity.started_at.to_rfc3339(),
                activity.ended_at.to_rfc3339(),
                activity.category.as_str(),
                activity.summary,
                activity.detail,
                activity.confidence,
                activity.is_work_related as i32,
                activity.source,
                activity.edited_at.map(|d| d.to_rfc3339()),
                activity.deleted_at.map(|d| d.to_rfc3339()),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_activities_for_date(&self, date: NaiveDate) -> Result<Vec<Activity>, String> {
        let start = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end = date.and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap());
        let start_utc = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
        let end_utc = DateTime::<Utc>::from_naive_utc_and_offset(end, Utc);

        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, started_at, ended_at, category, summary, detail, confidence, is_work_related, source, edited_at, deleted_at
                 FROM activities
                 WHERE deleted_at IS NULL
                   AND started_at >= ?1 AND started_at <= ?2
                 ORDER BY started_at",
            )
            .map_err(|e| e.to_string())?;

        let activities = stmt
            .query_map(
                params![start_utc.to_rfc3339(), end_utc.to_rfc3339()],
                row_to_activity,
            )
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(activities)
    }

    pub fn update_activity(&self, activity: &Activity) -> Result<(), String> {
        let conn = self.conn();
        let rows = conn.execute(
            "UPDATE activities SET job_id = ?2, started_at = ?3, ended_at = ?4, category = ?5, summary = ?6, detail = ?7, confidence = ?8, is_work_related = ?9, source = ?10, edited_at = ?11, deleted_at = ?12
             WHERE id = ?1",
            params![
                activity.id,
                activity.job_id,
                activity.started_at.to_rfc3339(),
                activity.ended_at.to_rfc3339(),
                activity.category.as_str(),
                activity.summary,
                activity.detail,
                activity.confidence,
                activity.is_work_related as i32,
                activity.source,
                activity.edited_at.map(|d| d.to_rfc3339()),
                activity.deleted_at.map(|d| d.to_rfc3339()),
            ],
        ).map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err(format!("activity {} not found", activity.id));
        }
        Ok(())
    }

    pub fn get_activity_by_id(&self, id: &str) -> Result<Option<Activity>, String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, started_at, ended_at, category, summary, detail, confidence, is_work_related, source, edited_at, deleted_at
                 FROM activities WHERE id = ?1 AND deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let mut rows = stmt
            .query_map(params![id], row_to_activity)
            .map_err(|e| e.to_string())?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    pub fn soft_delete_activity(&self, id: &str) -> Result<(), String> {
        let conn = self.conn();
        let now = Utc::now().to_rfc3339();
        let rows = conn
            .execute(
                "UPDATE activities SET deleted_at = ?2 WHERE id = ?1 AND deleted_at IS NULL",
                params![id, now],
            )
            .map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err(format!("activity {} not found or already deleted", id));
        }
        Ok(())
    }

    pub fn get_activities_in_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Activity>, String> {
        let start_dt = start.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let end_dt = end.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let start_utc = DateTime::<Utc>::from_naive_utc_and_offset(start_dt, Utc);
        let end_utc = DateTime::<Utc>::from_naive_utc_and_offset(end_dt, Utc);

        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, started_at, ended_at, category, summary, detail, confidence, is_work_related, source, edited_at, deleted_at
                 FROM activities
                 WHERE deleted_at IS NULL
                   AND started_at >= ?1 AND started_at < ?2
                 ORDER BY started_at",
            )
            .map_err(|e| e.to_string())?;

        let activities = stmt
            .query_map(
                params![start_utc.to_rfc3339(), end_utc.to_rfc3339()],
                row_to_activity,
            )
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(activities)
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

    fn insert_job(db: &Database, job_id: &str) {
        db.conn()
            .execute(
                "INSERT INTO analysis_jobs (id, captured_at, status) VALUES (?1, ?2, ?3)",
                params![job_id, Utc::now().to_rfc3339(), "done"],
            )
            .unwrap();
    }

    fn make_activity(job_id: &str, start_hour: u32, end_hour: u32) -> Activity {
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        Activity {
            id: Uuid::new_v4().to_string(),
            job_id: job_id.to_string(),
            started_at: Utc.from_utc_datetime(&date.and_hms_opt(start_hour, 0, 0).unwrap()),
            ended_at: Utc.from_utc_datetime(&date.and_hms_opt(end_hour, 0, 0).unwrap()),
            category: Category::Development,
            summary: "Coding session".to_string(),
            detail: Some("Working on Rust code".to_string()),
            confidence: 0.9,
            is_work_related: true,
            source: "auto".to_string(),
            edited_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let db = setup_db();
        let job_id = Uuid::new_v4().to_string();
        insert_job(&db, &job_id);

        let activity = make_activity(&job_id, 9, 10);
        db.insert_activity(&activity).unwrap();

        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let activities = db.get_activities_for_date(date).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].id, activity.id);
        assert_eq!(activities[0].summary, "Coding session");
        assert_eq!(activities[0].category, Category::Development);
    }

    #[test]
    fn test_soft_delete() {
        let db = setup_db();
        let job_id = Uuid::new_v4().to_string();
        insert_job(&db, &job_id);

        let activity = make_activity(&job_id, 9, 10);
        db.insert_activity(&activity).unwrap();

        db.soft_delete_activity(&activity.id).unwrap();

        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let activities = db.get_activities_for_date(date).unwrap();
        assert_eq!(activities.len(), 0);

        let err = db.soft_delete_activity(&activity.id);
        assert!(err.is_err());
    }

    #[test]
    fn test_update() {
        let db = setup_db();
        let job_id = Uuid::new_v4().to_string();
        insert_job(&db, &job_id);

        let mut activity = make_activity(&job_id, 9, 10);
        db.insert_activity(&activity).unwrap();

        activity.summary = "Updated summary".to_string();
        activity.confidence = 0.95;
        activity.edited_at = Some(Utc::now());
        db.update_activity(&activity).unwrap();

        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let activities = db.get_activities_for_date(date).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].summary, "Updated summary");
        assert!((activities[0].confidence - 0.95).abs() < f64::EPSILON);
        assert!(activities[0].edited_at.is_some());
    }

    #[test]
    fn test_get_activities_in_range() {
        let db = setup_db();
        let job_id = Uuid::new_v4().to_string();
        insert_job(&db, &job_id);

        let date1 = NaiveDate::from_ymd_opt(2025, 6, 14).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2025, 6, 16).unwrap();

        let mut a1 = make_activity(&job_id, 9, 10);
        a1.started_at = Utc.from_utc_datetime(&date1.and_hms_opt(9, 0, 0).unwrap());
        a1.ended_at = Utc.from_utc_datetime(&date1.and_hms_opt(10, 0, 0).unwrap());

        let mut a2 = make_activity(&job_id, 11, 12);
        a2.started_at = Utc.from_utc_datetime(&date2.and_hms_opt(11, 0, 0).unwrap());
        a2.ended_at = Utc.from_utc_datetime(&date2.and_hms_opt(12, 0, 0).unwrap());

        let mut a3 = make_activity(&job_id, 14, 15);
        a3.started_at = Utc.from_utc_datetime(&date3.and_hms_opt(14, 0, 0).unwrap());
        a3.ended_at = Utc.from_utc_datetime(&date3.and_hms_opt(15, 0, 0).unwrap());

        db.insert_activity(&a1).unwrap();
        db.insert_activity(&a2).unwrap();
        db.insert_activity(&a3).unwrap();

        // Range [date1, date3) should return a1 and a2
        let activities = db.get_activities_in_range(date1, date3).unwrap();
        assert_eq!(activities.len(), 2);
        assert_eq!(activities[0].id, a1.id);
        assert_eq!(activities[1].id, a2.id);

        // Range [date2, date3) should return only a2
        let activities = db.get_activities_in_range(date2, date3).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].id, a2.id);
    }
}
