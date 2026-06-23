use chrono::NaiveDate;

use crate::domain::PeriodType;

use super::aggregation::AggregatedReport;

pub fn export_markdown(
    report: &AggregatedReport,
    period_type: &PeriodType,
    period_start: NaiveDate,
    template_prompt: Option<&str>,
) -> String {
    let mut md = String::new();

    let period_label = match period_type {
        PeriodType::Daily => format!("Daily Report – {}", period_start),
        PeriodType::Weekly => format!("Weekly Report – {}", period_start),
        PeriodType::Custom => format!("Report – {}", period_start),
    };

    md.push_str(&format!("# {}\n\n", period_label));

    let work_h = report.work_minutes / 60;
    let work_m = report.work_minutes % 60;
    let total_h = report.total_minutes / 60;
    let total_m = report.total_minutes % 60;

    md.push_str(&format!(
        "**Total**: {}h {}m | **Work**: {}h {}m | **Activities**: {}\n\n",
        total_h, total_m, work_h, work_m, report.activity_count
    ));

    for cat in &report.categories {
        md.push_str(&format!("## {} ({}m)\n\n", cat.category, cat.total_minutes));

        for act in &cat.activities {
            md.push_str(&format!(
                "- {:02}:{:02}–{:02}:{:02} ({}m) — {}",
                act.start_hour, act.start_min, act.end_hour, act.end_min, act.minutes, act.summary
            ));
            if !act.is_work_related {
                md.push_str(" [personal]");
            }
            if let Some(ref detail) = act.detail {
                md.push_str(&format!("\n  {}", detail));
            }
            md.push('\n');
        }
        md.push('\n');
    }

    if let Some(prompt) = template_prompt {
        md.push_str("---\n\n");
        md.push_str(&format!("*Template guidance: {}*\n", prompt));
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Category;
    use crate::reporting::aggregation::{ActivitySummary, CategorySummary};

    fn make_report() -> AggregatedReport {
        AggregatedReport {
            categories: vec![CategorySummary {
                category: Category::Development,
                total_minutes: 120,
                activities: vec![
                    ActivitySummary {
                        summary: "Coding".to_string(),
                        detail: Some("Rust backend".to_string()),
                        start_hour: 9,
                        start_min: 0,
                        end_hour: 11,
                        end_min: 0,
                        minutes: 120,
                        is_work_related: true,
                        confidence: 0.95,
                    },
                ],
            }],
            total_minutes: 120,
            work_minutes: 120,
            activity_count: 1,
        }
    }

    #[test]
    fn test_export_contains_header_and_activities() {
        let report = make_report();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let md = export_markdown(&report, &PeriodType::Daily, date, None);

        assert!(md.contains("# Daily Report"));
        assert!(md.contains("## development"));
        assert!(md.contains("Coding"));
        assert!(md.contains("Rust backend"));
        assert!(md.contains("**Total**"));
        assert!(md.contains("**Work**"));
    }

    #[test]
    fn test_export_with_template_prompt() {
        let report = make_report();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let md = export_markdown(&report, &PeriodType::Daily, date, Some("Be concise"));

        assert!(md.contains("Template guidance: Be concise"));
    }
}
