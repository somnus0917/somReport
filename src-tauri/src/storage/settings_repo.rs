use super::Database;
use crate::domain::AppSettings;
use serde_json::Value;

const SETTINGS_KEY: &str = "app_settings";

fn migrate_provider_cost_units(provider: &mut Value) {
    let Some(provider) = provider.as_object_mut() else {
        return;
    };

    if !provider.contains_key("input_cost_per_million_yuan") {
        if let Some(cents) = provider
            .get("input_cost_per_million_cents")
            .and_then(Value::as_f64)
        {
            provider.insert(
                "input_cost_per_million_yuan".to_string(),
                Value::from(cents / 100.0),
            );
        }
    }
    if !provider.contains_key("output_cost_per_million_yuan") {
        if let Some(cents) = provider
            .get("output_cost_per_million_cents")
            .and_then(Value::as_f64)
        {
            provider.insert(
                "output_cost_per_million_yuan".to_string(),
                Value::from(cents / 100.0),
            );
        }
    }
}

fn migrate_settings_units(json: &str) -> Result<AppSettings, String> {
    let mut value: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let Some(settings) = value.as_object_mut() else {
        return serde_json::from_value(value).map_err(|e| e.to_string());
    };

    if !settings.contains_key("max_daily_cost_yuan") {
        if let Some(cents) = settings
            .get("max_daily_cost_cents")
            .and_then(Value::as_f64)
        {
            settings.insert("max_daily_cost_yuan".to_string(), Value::from(cents / 100.0));
        }
    }
    if let Some(provider) = settings.get_mut("vision_provider") {
        migrate_provider_cost_units(provider);
    }
    if let Some(provider) = settings.get_mut("text_provider") {
        migrate_provider_cost_units(provider);
    }

    serde_json::from_value(value).map_err(|e| e.to_string())
}

impl Database {
    pub fn get_settings(&self) -> Result<AppSettings, String> {
        let conn = self.conn();
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![SETTINGS_KEY],
            |row| row.get(0),
        );

        match result {
            Ok(json) => migrate_settings_units(&json),
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

        assert_eq!(
            settings.capture_interval_secs,
            defaults.capture_interval_secs
        );
        assert_eq!(settings.idle_timeout_secs, defaults.idle_timeout_secs);
        assert_eq!(settings.max_daily_cost_yuan, defaults.max_daily_cost_yuan);
        assert_eq!(settings.auto_start, defaults.auto_start);
        assert_eq!(settings.vision_provider.name, defaults.vision_provider.name);
    }

    #[test]
    fn test_save_and_get_settings() {
        let db = setup_db();

        let mut settings = AppSettings::default();
        settings.capture_interval_secs = 60;
        settings.auto_start = true;
        settings.max_daily_cost_yuan = 10.0;
        settings.vision_provider.model = "gpt-4o".to_string();

        db.save_settings(&settings).unwrap();

        let loaded = db.get_settings().unwrap();
        assert_eq!(loaded.capture_interval_secs, 60);
        assert!(loaded.auto_start);
        assert_eq!(loaded.max_daily_cost_yuan, 10.0);
        assert_eq!(loaded.vision_provider.model, "gpt-4o");
    }

    #[test]
    fn test_loads_settings_created_before_connection_statuses_existed() {
        let db = setup_db();
        let old_settings = r#"{
          "vision_provider":{"name":"qwen","api_key_env_var":"QWEN_API_KEY","api_key":null,"api_url":"https://example.test","model":"qwen-vl-max"},
          "text_provider":{"name":"qwen","api_key_env_var":"QWEN_API_KEY","api_key":null,"api_url":"https://example.test","model":"qwen-plus"},
          "capture_interval_secs":30,
          "idle_timeout_secs":300,
          "max_daily_cost_cents":500,
          "auto_start":false,
          "notify_on_report":true
        }"#;
        db.conn()
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                rusqlite::params![SETTINGS_KEY, old_settings],
            )
            .unwrap();

        let loaded = db.get_settings().unwrap();
        assert_eq!(loaded.vision_connection.success, None);
        assert_eq!(loaded.text_connection.tested_at, None);
        assert_eq!(loaded.max_daily_cost_yuan, 5.0);
    }
}
