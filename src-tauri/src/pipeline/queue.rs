use std::sync::Arc;

use chrono::Utc;
use md5::{Digest, Md5};
use uuid::Uuid;

use crate::domain::{Activity, AnalysisJob, CapturedFrame, JobStatus, VisionProvider, VisionResult};
use crate::pipeline::dedup::DedupChecker;
use crate::pipeline::retry::{with_retry, RetryConfig};
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
            Ok(result) => {
                let activities = self.create_activities(&job, frame, &result);
                for activity in &activities {
                    self.db.insert_activity(activity)?;
                }
                self.db.complete_job(&job.id, provider_name, model_name)?;
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
    ) -> Vec<Activity> {
        result
            .items
            .iter()
            .map(|item| Activity {
                id: Uuid::new_v4().to_string(),
                job_id: job.id.clone(),
                started_at: frame.captured_at,
                ended_at: frame.captured_at,
                category: item.category.clone(),
                summary: item.summary.clone(),
                detail: item.detail.clone(),
                confidence: item.confidence,
                is_work_related: item.is_work_related,
                source: "auto".to_string(),
                edited_at: None,
                deleted_at: None,
            })
            .collect()
    }
}

fn compute_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
