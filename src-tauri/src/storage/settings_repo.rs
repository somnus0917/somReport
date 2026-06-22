use crate::domain::AppSettings;
use super::Database;

const SETTINGS_KEY: &str = "app_settings";

impl Database {
    pub fn get_settings(&self) -> Result<AppSettings, String> {
        let conn = self.conn();
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![SETTINGS_KEY],
            |row| row.get(0),
        );

        match result {
            Ok(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(|e| e.to_string())?;
        let conn = self.conn();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
            rusqlite::params![SETTINGS_KEY, json],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Database {
        Database::new_in_memory().expect("failed to create in-memory db")
    }

    #[test]
    fn test_get_settings_returns_defaults_when_empty() {
        let db = setup_db();
        let settings = db.get_settings().unwrap();
        let defaults = AppSettings::default();

        assert_eq!(settings.capture_interval_secs, defaults.capture_interval_secs);
        assert_eq!(settings.idle_timeout_secs, defaults.idle_timeout_secs);
        assert_eq!(settings.max_daily_cost_cents, defaults.max_daily_cost_cents);
        assert_eq!(settings.auto_start, defaults.auto_start);
        assert_eq!(settings.vision_provider.name, defaults.vision_provider.name);
    }

    #[test]
    fn test_save_and_get_settings() {
        let db = setup_db();

        let mut settings = AppSettings::default();
        settings.capture_interval_secs = 60;
        settings.auto_start = true;
        settings.max_daily_cost_cents = 1000;
        settings.vision_provider.model = "gpt-4o".to_string();

        db.save_settings(&settings).unwrap();

        let loaded = db.get_settings().unwrap();
        assert_eq!(loaded.capture_interval_secs, 60);
        assert!(loaded.auto_start);
        assert_eq!(loaded.max_daily_cost_cents, 1000);
        assert_eq!(loaded.vision_provider.model, "gpt-4o");
    }
}
