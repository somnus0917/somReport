use rusqlite::Connection;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", include_str!("../migrations/001_initial.sql")),
];

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );",
    )?;

    let applied: Vec<String> = {
        let mut stmt = conn.prepare("SELECT name FROM _migrations")?;
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0))?.collect::<Result<_, _>>()?;
        rows
    };

    for (name, sql) in MIGRATIONS {
        if applied.contains(&name.to_string()) {
            continue;
        }
        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO _migrations (name) VALUES (?1)",
            rusqlite::params![name],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_run_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let tables: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '\\_%' ESCAPE '\\' ORDER BY name",
                )
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };

        let expected = [
            "activities",
            "analysis_jobs",
            "api_usage",
            "capture_sessions",
            "reports",
            "settings",
        ];
        assert_eq!(tables, expected);
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM _migrations",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
