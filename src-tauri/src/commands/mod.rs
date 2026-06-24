use chrono::{Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::domain::{
    Activity, AppSettings, CapturedFrame, ModelConnectionStatus, PeriodType, ProviderConfig, Report,
};
use crate::pipeline::scheduler::CaptureScheduler;
use crate::platform::idle::IdleDetector;
use crate::providers;
use crate::reporting::{aggregation, export, templates};
use crate::storage::usage_repo::UsageEntry;
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
pub async fn generate_report(
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

    let activities =
        db.get_activities_in_range(start_date, end_date + chrono::Duration::days(1))?;
    let agg = aggregation::aggregate_activities(&activities);

    let template_prompt = template_id
        .as_deref()
        .and_then(templates::get_template_prompt);

    let local_content = export::export_markdown(&agg, &pt, start_date, template_prompt);
    let settings = db.get_settings()?;
    let (content, model) = match providers::create_text_provider(&settings.text_provider) {
        Ok(provider) => match provider
            .generate(&format!(
                "Create a polished report from this locally aggregated activity data. Preserve factual times and do not invent activity.\n\n{local_content}"
            ))
            .await
        {
            Ok(response) if !response.value.trim().is_empty() => {
                db.record_usage(&UsageEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    occurred_at: Utc::now(),
                    provider: settings.text_provider.name.clone(),
                    model: settings.text_provider.model.clone(),
                    input_tokens: response.usage.input_tokens,
                    output_tokens: response.usage.output_tokens,
                    estimated_cost_cents: estimate_cost_cents(
                        response.usage.input_tokens,
                        response.usage.output_tokens,
                        settings.text_provider.input_cost_per_million_cents,
                        settings.text_provider.output_cost_per_million_cents,
                    ),
                    job_id: None,
                })?;
                (response.value, Some(settings.text_provider.model))
            }
            Ok(_) | Err(_) => (local_content, None),
        },
        Err(_) => (local_content, None),
    };

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
        period_end: end_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        template_id,
        title,
        content_markdown: content,
        model,
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
pub fn start_recording(
    app: AppHandle,
    db: State<'_, Database>,
    scheduler: State<'_, CaptureScheduler>,
) -> Result<(), String> {
    let settings = db.get_settings()?;
    // Fail before changing the UI state if recording cannot reach a configured provider.
    providers::create_vision_provider(&settings.vision_provider)?;
    scheduler.start();
    app.emit("recording-status", "recording")
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn pause_recording(
    app: AppHandle,
    scheduler: State<'_, CaptureScheduler>,
) -> Result<(), String> {
    scheduler.pause();
    app.emit("recording-status", "paused")
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn stop_recording(
    app: AppHandle,
    scheduler: State<'_, CaptureScheduler>,
) -> Result<(), String> {
    scheduler.stop();
    app.emit("recording-status", "stopped")
        .map_err(|e| e.to_string())?;
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
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_settings(db: State<'_, Database>) -> Result<AppSettings, String> {
    db.get_settings()
}

#[tauri::command]
pub fn save_settings(
    db: State<'_, Database>,
    scheduler: State<'_, CaptureScheduler>,
    idle_detector: State<'_, IdleDetector>,
    settings: AppSettings,
) -> Result<(), String> {
    validate_settings(&settings)?;
    db.save_settings(&settings)?;
    db.purge_records_older_than(settings.data_retention_days)?;
    scheduler.set_interval(settings.capture_interval_secs as u64);
    idle_detector.set_threshold(settings.idle_timeout_secs as u64);
    Ok(())
}

#[tauri::command]
pub fn clear_all_data(db: State<'_, Database>) -> Result<(), String> {
    {
        let conn = db.conn();
        conn.execute_batch(
            "DELETE FROM activities;
             DELETE FROM analysis_jobs;
             DELETE FROM api_usage;
             DELETE FROM reports;
             DELETE FROM capture_sessions;
             DELETE FROM settings;",
        )
        .map_err(|e| e.to_string())?;
    }
    db.checkpoint_wal()?;
    crate::platform::paths::cleanup_temp_files().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cleanup_local_storage(db: State<'_, Database>, retention_days: u32) -> Result<(), String> {
    db.purge_records_older_than(retention_days)?;
    db.checkpoint_wal()?;
    crate::platform::paths::clear_cache().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_daily_usage(
    db: State<'_, Database>,
) -> Result<crate::storage::usage_repo::DailyUsage, String> {
    let today = Local::now().date_naive();
    db.get_daily_usage(today)
}

fn validate_settings(settings: &AppSettings) -> Result<(), String> {
    if !(5..=3600).contains(&settings.capture_interval_secs) {
        return Err("Capture interval must be between 5 seconds and 1 hour".to_string());
    }
    if !(30..=86_400).contains(&settings.idle_timeout_secs) {
        return Err("Idle timeout must be between 30 seconds and 24 hours".to_string());
    }
    for (label, provider) in [
        ("vision", &settings.vision_provider),
        ("text", &settings.text_provider),
    ] {
        if !matches!(provider.name.as_str(), "openai" | "anthropic" | "qwen") {
            return Err(format!("Unsupported {label} provider: {}", provider.name));
        }
        if provider.model.trim().is_empty() || provider.api_url.trim().is_empty() {
            return Err(format!("{label} provider model and API URL are required"));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub response: Option<String>,
}

#[tauri::command]
pub async fn test_model_connection(
    provider_type: String,
    config: ProviderConfig,
) -> Result<TestResult, String> {
    Ok(test_model_config(&provider_type, &config).await)
}

#[tauri::command]
pub fn save_model_config(
    db: State<'_, Database>,
    provider_type: String,
    config: ProviderConfig,
) -> Result<AppSettings, String> {
    if !matches!(provider_type.as_str(), "vision" | "text") {
        return Err("Invalid provider type".to_string());
    }
    validate_provider_config(&config)?;
    let mut settings = db.get_settings()?;
    update_model_config(&mut settings, &provider_type, config)?;
    db.save_settings(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn save_and_test_model(
    db: State<'_, Database>,
    provider_type: String,
    config: ProviderConfig,
) -> Result<TestResult, String> {
    let settings = save_model_config(db.clone(), provider_type.clone(), config)?;
    let active_config = match provider_type.as_str() {
        "vision" => settings.vision_provider,
        "text" => settings.text_provider,
        _ => return Err("Invalid provider type".to_string()),
    };
    let result = test_model_config(&provider_type, &active_config).await;

    let mut saved_settings = db.get_settings()?;
    update_connection_status(&mut saved_settings, &provider_type, &result)?;
    db.save_settings(&saved_settings)?;
    Ok(result)
}

async fn test_model_config(provider_type: &str, config: &ProviderConfig) -> TestResult {
    if !matches!(provider_type, "vision" | "text") {
        return TestResult {
            success: false,
            message: "无效的模型类型".to_string(),
            response: None,
        };
    }

    log::info!(
        "Testing model connection: provider={}, model={}, url={}",
        config.name,
        config.model,
        config.api_url
    );

    // Resolve API key
    if let Err(e) = providers::resolve_api_key(config) {
        log::error!("Failed to resolve API key: {e}");
        return TestResult {
            success: false,
            message: format!("API 密钥未配置: {e}"),
            response: None,
        };
    }

    let result = match provider_type {
        "text" => {
            let provider = providers::create_text_provider(config);
            match provider {
                Ok(provider) => provider
                    .generate("Reply only with: connection ok")
                    .await
                    .map(|response| response.value),
                Err(error) => Err(error),
            }
        }
        "vision" => {
            let frame = match model_test_frame() {
                Ok(frame) => frame,
                Err(error) => {
                    return TestResult {
                        success: false,
                        message: format!("无法创建视觉测试图片: {error}"),
                        response: None,
                    };
                }
            };
            let provider = providers::create_vision_provider(config);
            match provider {
                Ok(provider) => provider.analyze(&frame).await.and_then(|response| {
                    serde_json::to_string_pretty(&response.value).map_err(|error| error.to_string())
                }),
                Err(error) => Err(error),
            }
        }
        _ => unreachable!(),
    };

    match result {
        Ok(response) => {
            log::info!("Model test successful, response length: {}", response.len());
            TestResult {
                success: true,
                message: format!("{} ({}) 连接成功", config.name, config.model),
                response: Some(response),
            }
        }
        Err(e) => {
            log::error!("Model test failed: {e}");
            TestResult {
                success: false,
                message: format!("{} ({}) 连接失败: {e}", config.name, config.model),
                response: None,
            }
        }
    }
}

fn update_model_config(
    settings: &mut AppSettings,
    provider_type: &str,
    config: ProviderConfig,
) -> Result<(), String> {
    match provider_type {
        "vision" => {
            settings.vision_provider = config;
            settings.vision_connection = ModelConnectionStatus::default();
        }
        "text" => {
            settings.text_provider = config;
            settings.text_connection = ModelConnectionStatus::default();
        }
        _ => return Err("Invalid provider type".to_string()),
    }
    Ok(())
}

fn update_connection_status(
    settings: &mut AppSettings,
    provider_type: &str,
    result: &TestResult,
) -> Result<(), String> {
    let status = ModelConnectionStatus {
        success: Some(result.success),
        tested_at: Some(Utc::now().to_rfc3339()),
        message: Some(result.message.clone()),
    };
    match provider_type {
        "vision" => settings.vision_connection = status,
        "text" => settings.text_connection = status,
        _ => return Err("Invalid provider type".to_string()),
    }
    Ok(())
}

fn validate_provider_config(config: &ProviderConfig) -> Result<(), String> {
    if !matches!(config.name.as_str(), "openai" | "anthropic" | "qwen") {
        return Err(format!("Unsupported provider: {}", config.name));
    }
    if config.model.trim().is_empty() || config.api_url.trim().is_empty() {
        return Err("模型名称和 API 地址不能为空".to_string());
    }
    Ok(())
}

fn estimate_cost_cents(
    input_tokens: i64,
    output_tokens: i64,
    input_cost_per_million_cents: f64,
    output_cost_per_million_cents: f64,
) -> f64 {
    (input_tokens as f64 * input_cost_per_million_cents
        + output_tokens as f64 * output_cost_per_million_cents)
        / 1_000_000.0
}

fn model_test_frame() -> Result<CapturedFrame, String> {
    let image = image::RgbaImage::from_pixel(32, 32, image::Rgba([75, 108, 183, 255]));
    let mut encoded = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .map_err(|error| format!("Failed to create vision test image: {error}"))?;

    Ok(CapturedFrame {
        id: "model-connection-test".to_string(),
        captured_at: Utc::now(),
        png_data: encoded.into_inner(),
        mime_type: "image/png".to_string(),
        width: 32,
        height: 32,
        display_index: 0,
        image_hash: None,
    })
}
