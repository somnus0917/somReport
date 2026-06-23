use std::collections::HashMap;

use chrono::Timelike;

use crate::domain::{Activity, Category};

#[derive(Debug, Clone)]
pub struct CategorySummary {
    pub category: Category,
    pub total_minutes: i64,
    pub activities: Vec<ActivitySummary>,
}

#[derive(Debug, Clone)]
pub struct ActivitySummary {
    pub summary: String,
    pub detail: Option<String>,
    pub start_hour: u32,
    pub start_min: u32,
    pub end_hour: u32,
    pub end_min: u32,
    pub minutes: i64,
    pub is_work_related: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct AggregatedReport {
    pub categories: Vec<CategorySummary>,
    pub total_minutes: i64,
    pub work_minutes: i64,
    pub activity_count: usize,
}

pub fn aggregate_activities(activities: &[Activity]) -> AggregatedReport {
    let mut by_category: HashMap<Category, Vec<&Activity>> = HashMap::new();
    let mut total_minutes: i64 = 0;
    let mut work_minutes: i64 = 0;

    for a in activities {
        let mins = (a.ended_at - a.started_at).num_minutes().max(0);
        total_minutes += mins;
        if a.is_work_related {
            work_minutes += mins;
        }
        by_category.entry(a.category.clone()).or_default().push(a);
    }

    let order = [
        Category::Development,
        Category::Meeting,
        Category::Communication,
        Category::Documentation,
        Category::Research,
        Category::Design,
        Category::Other,
    ];

    let mut categories = Vec::new();
    for cat in &order {
        if let Some(acts) = by_category.get(cat) {
            let mut summaries: Vec<ActivitySummary> = acts
                .iter()
                .map(|a| {
                    let mins = (a.ended_at - a.started_at).num_minutes().max(0);
                    ActivitySummary {
                        summary: a.summary.clone(),
                        detail: a.detail.clone(),
                        start_hour: a.started_at.hour(),
                        start_min: a.started_at.minute(),
                        end_hour: a.ended_at.hour(),
                        end_min: a.ended_at.minute(),
                        minutes: mins,
                        is_work_related: a.is_work_related,
                        confidence: a.confidence,
                    }
                })
                .collect();

            summaries.sort_by(|a, b| {
                (a.start_hour * 60 + a.start_min).cmp(&(b.start_hour * 60 + b.start_min))
            });

            let cat_total: i64 = summaries.iter().map(|s| s.minutes).sum();

            categories.push(CategorySummary {
                category: cat.clone(),
                total_minutes: cat_total,
                activities: summaries,
            });
        }
    }

    AggregatedReport {
        categories,
        total_minutes,
        work_minutes,
        activity_count: activities.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};
    use uuid::Uuid;

    fn make_activity(
        category: Category,
        summary: &str,
        start_hour: u32,
        end_hour: u32,
        is_work: bool,
    ) -> Activity {
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        Activity {
            id: Uuid::new_v4().to_string(),
            job_id: "test-job".to_string(),
            started_at: Utc.from_utc_datetime(&date.and_hms_opt(start_hour, 0, 0).unwrap()),
            ended_at: Utc.from_utc_datetime(&date.and_hms_opt(end_hour, 0, 0).unwrap()),
            category,
            summary: summary.to_string(),
            detail: None,
            confidence: 0.9,
            is_work_related: is_work,
            source: "auto".to_string(),
            edited_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_aggregate_groups_by_category() {
        let activities = vec![
            make_activity(Category::Development, "Coding", 9, 11, true),
            make_activity(Category::Meeting, "Standup", 11, 12, true),
            make_activity(Category::Development, "Review PR", 13, 14, true),
            make_activity(Category::Research, "Read docs", 14, 15, false),
        ];

        let report = aggregate_activities(&activities);

        assert_eq!(report.activity_count, 4);
        assert_eq!(report.total_minutes, 300);
        assert_eq!(report.work_minutes, 240);
        assert_eq!(report.categories.len(), 3);

        let dev = report.categories.iter().find(|c| c.category == Category::Development).unwrap();
        assert_eq!(dev.activities.len(), 2);
        assert_eq!(dev.total_minutes, 180);

        let meeting = report.categories.iter().find(|c| c.category == Category::Meeting).unwrap();
        assert_eq!(meeting.activities.len(), 1);
        assert_eq!(meeting.total_minutes, 60);
    }

    #[test]
    fn test_aggregate_empty_activities() {
        let activities: Vec<Activity> = vec![];
        let report = aggregate_activities(&activities);

        assert_eq!(report.activity_count, 0);
        assert_eq!(report.total_minutes, 0);
        assert_eq!(report.work_minutes, 0);
        assert!(report.categories.is_empty());
    }
}
