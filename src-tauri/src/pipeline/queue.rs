use std::sync::Arc;

use chrono::Utc;
use md5::{Digest, Md5};
use uuid::Uuid;

use crate::domain::{
    Activity, AnalysisJob, CapturedFrame, JobStatus, VisionProvider, VisionResult,
};
use crate::pipeline::dedup::DedupChecker;
use crate::pipeline::retry::{with_retry, RetryConfig};
use crate::storage::usage_repo::UsageEntry;
use crate::storage::Database;

pub struct QueueWorker {
    pub db: Arc<Database>,
    pub dedup: DedupChecker,
}

impl QueueWorker {
    pub fn new(db: Arc<Database>, dedup_threshold: f64) -> Self {
        Self {
            db,
            dedup: DedupChecker::new(dedup_threshold),
        }
    }

    pub async fn process_frame(
        &mut self,
        frame: &CapturedFrame,
        provider: &dyn VisionProvider,
        provider_name: &str,
        model_name: &str,
        input_cost_per_million_cents: f64,
        output_cost_per_million_cents: f64,
        activity_window_secs: u64,
    ) -> Result<Vec<Activity>, String> {
        let is_dup = self.dedup.check_and_update(&frame.png_data)?;
        if is_dup {
            log::debug!("Skipping duplicate frame {}", frame.id);
            return Ok(vec![]);
        }

        let image_hash = compute_md5(&frame.png_data);

        let job = AnalysisJob {
            id: Uuid::new_v4().to_string(),
            captured_at: frame.captured_at,
            status: JobStatus::Pending,
            attempts: 0,
            last_error: None,
            image_hash: Some(image_hash),
            provider: Some(provider_name.to_string()),
            model: Some(model_name.to_string()),
            created_at: Utc::now(),
            finished_at: None,
        };
        self.db.insert_job(&job)?;

        let retry_config = RetryConfig::default();
        let vision_result = with_retry(&retry_config, || provider.analyze(frame)).await;

        match vision_result {
            Ok(response) => {
                let activities =
                    self.create_activities(&job, frame, &response.value, activity_window_secs);
                for activity in &activities {
                    self.db.insert_activity(activity)?;
                }
                self.db.complete_job(&job.id, provider_name, model_name)?;
                self.db.record_usage(&UsageEntry {
                    id: Uuid::new_v4().to_string(),
                    occurred_at: Utc::now(),
                    provider: provider_name.to_string(),
                    model: model_name.to_string(),
                    input_tokens: response.usage.input_tokens,
                    output_tokens: response.usage.output_tokens,
                    estimated_cost_cents: estimate_cost_cents(
                        response.usage.input_tokens,
                        response.usage.output_tokens,
                        input_cost_per_million_cents,
                        output_cost_per_million_cents,
                    ),
                    job_id: Some(job.id.clone()),
                })?;
                Ok(activities)
            }
            Err(e) => {
                log::error!("Vision analysis failed for job {}: {}", job.id, e);
                self.db.fail_job(&job.id, &e)?;
                Err(e)
            }
        }
    }

    fn create_activities(
        &self,
        job: &AnalysisJob,
        frame: &CapturedFrame,
        result: &VisionResult,
        activity_window_secs: u64,
    ) -> Vec<Activity> {
        let window_start =
            frame.captured_at - chrono::Duration::seconds(activity_window_secs as i64);
        let item_count = result.items.len() as i64;
        result
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let start_offset = activity_window_secs as i64 * index as i64 / item_count;
                let end_offset = activity_window_secs as i64 * (index as i64 + 1) / item_count;
                Activity {
                    id: Uuid::new_v4().to_string(),
                    job_id: job.id.clone(),
                    // A frame describes the interval since the preceding capture, not an
                    // instantaneous event. Split it between distinct activities so totals
                    // never exceed the tracked time window.
                    started_at: window_start + chrono::Duration::seconds(start_offset),
                    ended_at: window_start + chrono::Duration::seconds(end_offset),
                    category: item.category.clone(),
                    summary: item.summary.clone(),
                    detail: item.detail.clone(),
                    confidence: item.confidence,
                    is_work_related: item.is_work_related,
                    source: "auto".to_string(),
                    edited_at: None,
                    deleted_at: None,
                }
            })
            .collect()
    }
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

fn compute_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Category, VisionItem};

    #[test]
    fn compute_md5_produces_hex_string() {
        let hash = compute_md5(b"hello world");
        assert_eq!(hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn compute_md5_empty_input() {
        let hash = compute_md5(b"");
        assert_eq!(hash, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn activities_split_a_capture_window_without_overcounting() {
        let worker = QueueWorker::new(Arc::new(Database::new_in_memory().unwrap()), 0.98);
        let captured_at = Utc::now();
        let frame = CapturedFrame {
            id: "frame".to_string(),
            captured_at,
            png_data: vec![],
            mime_type: "image/png".to_string(),
            width: 1,
            height: 1,
            display_index: 0,
            image_hash: None,
        };
        let job = AnalysisJob {
            id: "job".to_string(),
            captured_at,
            status: JobStatus::Pending,
            attempts: 0,
            last_error: None,
            image_hash: None,
            provider: None,
            model: None,
            created_at: captured_at,
            finished_at: None,
        };
        let result = VisionResult {
            items: vec![
                VisionItem {
                    category: Category::Development,
                    summary: "Coding".to_string(),
                    detail: None,
                    confidence: 1.0,
                    is_work_related: true,
                },
                VisionItem {
                    category: Category::Communication,
                    summary: "Replying".to_string(),
                    detail: None,
                    confidence: 1.0,
                    is_work_related: true,
                },
            ],
        };

        let activities = worker.create_activities(&job, &frame, &result, 30);
        assert_eq!(activities.len(), 2);
        assert_eq!(
            (activities[0].ended_at - activities[0].started_at).num_seconds(),
            15
        );
        assert_eq!(
            (activities[1].ended_at - activities[1].started_at).num_seconds(),
            15
        );
        assert_eq!(
            activities[0].started_at,
            captured_at - chrono::Duration::seconds(30)
        );
        assert_eq!(activities[1].ended_at, captured_at);
    }

    #[test]
    fn cost_estimation_preserves_fractional_cents() {
        let cost = estimate_cost_cents(1_000, 500, 15.0, 60.0);
        assert!((cost - 0.045).abs() < f64::EPSILON);
    }
}
