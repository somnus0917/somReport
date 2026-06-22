use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Development,
    Meeting,
    Communication,
    Documentation,
    Research,
    Design,
    Other,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Meeting => "meeting",
            Self::Communication => "communication",
            Self::Documentation => "documentation",
            Self::Research => "research",
            Self::Design => "design",
            Self::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "development" => Self::Development,
            "meeting" => Self::Meeting,
            "communication" => Self::Communication,
            "documentation" => Self::Documentation,
            "research" => Self::Research,
            "design" => Self::Design,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub job_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub category: Category,
    pub summary: String,
    pub detail: Option<String>,
    pub confidence: f64,
    pub is_work_related: bool,
    pub source: String,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Activity {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
    }
}
