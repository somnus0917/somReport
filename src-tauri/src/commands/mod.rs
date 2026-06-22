use chrono::{Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::domain::{Activity, AppSettings, PeriodType, Report};
use crate::pipeline::scheduler::CaptureScheduler;
use crate::reporting::{aggregation, export, templates};
use crate::storage::Database;

#[derive(Debug, Clone, Serialize)]
pub struct TodayStats {
    pub total_minutes: i64,
    pub work_minutes: i64,
    pub activity_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityUpdateRequest {
    pub id: String,
    pub summary: Option<String>,
    pub detail: Option<Option<String>>,
    pub category: Option<String>,
    pub is_work_related: Option<bool>,
    pub confidence: Option<f64>,
}

#[tauri::command]
pub fn get_today(db: State<'_, Database>) -> Result<(Vec<Activity>, TodayStats), String> {
    let today = Local::now().date_naive();
    let activities = db.get_activities_for_date(today)?;

    let total_minutes: i64 = activities
        .iter()
        .map(|a| (a.ended_at - a.started_at).num_minutes())
        .sum();
    let work_minutes: i64 = activities
        .iter()
        .filter(|a| a.is_work_related)
        .map(|a| (a.ended_at - a.started_at).num_minutes())
        .sum();

    let stats = TodayStats {
        total_minutes,
        work_minutes,
        activity_count: activities.len(),
    };

    Ok((activities, stats))
}

#[tauri::command]
pub fn update_activity(
    db: State<'_, Database>,
    request: ActivityUpdateRequest,
) -> Result<(), String> {
    let mut activity = db
        .get_activity_by_id(&request.id)?
        .ok_or_else(|| format!("activity {} not found", request.id))?;

    if let Some(summary) = request.summary {
        activity.summary = summary;
    }
    if let Some(detail) = request.detail {
        activity.detail = detail;
    }
    if let Some(is_work) = request.is_work_related {
        activity.is_work_related = is_work;
    }
    if let Some(confidence) = request.confidence {
        activity.confidence = confidence;
    }
    if let Some(cat_str) = request.category {
        activity.category = crate::domain::Category::from_str(&cat_str);
    }

    activity.edited_at = Some(Utc::now());
    db.update_activity(&activity)
}

#[tauri::command]
pub fn delete_activity(db: State<'_, Database>, id: String) -> Result<(), String> {
    db.soft_delete_activity(&id)
}

#[tauri::command]
pub fn generate_report(
    db: State<'_, Database>,
    period_type: String,
    period_start: String,
    template_id: Option<String>,
) -> Result<Report, String> {
    let pt = PeriodType::from_str(&period_type);
    let start_date = period_start
        .parse::<NaiveDate>()
        .map_err(|e| format!("invalid period_start: {e}"))?;

    let end_date = match pt {
        PeriodType::Daily => start_date,
        PeriodType::Weekly => start_date + chrono::Duration::days(6),
        PeriodType::Custom => start_date + chrono::Duration::days(6),
    };

    let activities = db.get_activities_in_range(start_date, end_date + chrono::Duration::days(1))?;
    let agg = aggregation::aggregate_activities(&activities);

    let template_prompt = template_id
        .as_deref()
        .and_then(templates::get_template_prompt);

    let content = export::export_markdown(&agg, &pt, start_date, template_prompt);

    let title = match pt {
        PeriodType::Daily => format!("Daily Report – {}", start_date),
        PeriodType::Weekly => format!("Weekly Report – {} to {}", start_date, end_date),
        PeriodType::Custom => format!("Report – {} to {}", start_date, end_date),
    };

    let now = Utc::now();
    let report = Report {
        id: uuid::Uuid::new_v4().to_string(),
        period_type: pt,
        period_start: start_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        period_end: end_date
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc(),
        template_id,
        title,
        content_markdown: content,
        model: None,
        prompt_version: Some("v1".to_string()),
        created_at: now,
        updated_at: now,
    };

    db.insert_report(&report)?;
    Ok(report)
}

#[tauri::command]
pub fn list_reports(
    db: State<'_, Database>,
    period_type: Option<String>,
) -> Result<Vec<Report>, String> {
    let pt = period_type.map(|s| PeriodType::from_str(&s));
    db.get_reports(pt)
}

#[tauri::command]
pub fn start_recording(scheduler: State<'_, CaptureScheduler>) -> Result<(), String> {
    scheduler.start();
    Ok(())
}

#[tauri::command]
pub fn pause_recording(scheduler: State<'_, CaptureScheduler>) -> Result<(), String> {
    scheduler.pause();
    Ok(())
}

#[tauri::command]
pub fn stop_recording(scheduler: State<'_, CaptureScheduler>) -> Result<(), String> {
    scheduler.stop();
    Ok(())
}

#[tauri::command]
pub fn get_recording_state(scheduler: State<'_, CaptureScheduler>) -> Result<String, String> {
    let state = scheduler.state();
    Ok(match state {
        crate::pipeline::scheduler::RecordingState::Stopped => "stopped".to_string(),
        crate::pipeline::scheduler::RecordingState::Recording => "recording".to_string(),
        crate::pipeline::scheduler::RecordingState::Paused => "paused".to_string(),
    })
}

#[tauri::command]
pub fn save_provider_key(service: String, key: String) -> Result<(), String> {
    let entry = keyring::Entry::new("daytrace", &service)
        .map_err(|e| format!("keyring error: {e}"))?;
    entry
        .set_password(&key)
        .map_err(|e| format!("failed to save key: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn test_provider_key(service: String) -> Result<bool, String> {
    let entry = keyring::Entry::new("daytrace", &service)
        .map_err(|e| format!("keyring error: {e}"))?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("keyring error: {e}")),
    }
}

#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<AppSettings, String> {
    db.get_settings()
}

#[tauri::command]
pub fn save_settings(db: State<'_, Database>, settings: AppSettings) -> Result<(), String> {
    db.save_settings(&settings)
}

#[tauri::command]
pub fn clear_all_data(db: State<'_, Database>) -> Result<(), String> {
    let conn = db.conn();
    conn.execute_batch(
        "DELETE FROM activities;
         DELETE FROM analysis_jobs;
         DELETE FROM api_usage;
         DELETE FROM reports;
         DELETE FROM capture_sessions;",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_daily_usage(db: State<'_, Database>) -> Result<i64, String> {
    let today = Local::now().date_naive();
    db.get_daily_usage_cents(today)
}
