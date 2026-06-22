CREATE TABLE IF NOT EXISTS settings (
    key                 TEXT PRIMARY KEY,
    value               TEXT NOT NULL,
    updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS capture_sessions (
    id                  TEXT PRIMARY KEY,
    started_at          TEXT NOT NULL,
    stopped_at          TEXT,
    backend             TEXT NOT NULL,
    display_config_json TEXT
);

CREATE TABLE IF NOT EXISTS analysis_jobs (
    id                  TEXT PRIMARY KEY,
    captured_at         TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending',
    attempts            INTEGER NOT NULL DEFAULT 0,
    last_error          TEXT,
    image_hash          TEXT,
    provider            TEXT,
    model               TEXT,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    finished_at         TEXT
);

CREATE TABLE IF NOT EXISTS activities (
    id                  TEXT PRIMARY KEY,
    job_id              TEXT NOT NULL REFERENCES analysis_jobs(id),
    started_at          TEXT NOT NULL,
    ended_at            TEXT NOT NULL,
    category            TEXT NOT NULL,
    summary             TEXT NOT NULL,
    detail              TEXT,
    confidence          REAL NOT NULL,
    is_work_related     INTEGER NOT NULL DEFAULT 1,
    source              TEXT NOT NULL DEFAULT 'auto',
    edited_at           TEXT,
    deleted_at          TEXT
);

CREATE TABLE IF NOT EXISTS reports (
    id                  TEXT PRIMARY KEY,
    period_type         TEXT NOT NULL,
    period_start        TEXT NOT NULL,
    period_end          TEXT NOT NULL,
    template_id         TEXT,
    title               TEXT NOT NULL,
    content_markdown    TEXT NOT NULL,
    model               TEXT,
    prompt_version      TEXT,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS api_usage (
    id                  TEXT PRIMARY KEY,
    occurred_at         TEXT NOT NULL,
    provider            TEXT NOT NULL,
    model               TEXT NOT NULL,
    input_tokens        INTEGER NOT NULL DEFAULT 0,
    output_tokens       INTEGER NOT NULL DEFAULT 0,
    estimated_cost_cents INTEGER NOT NULL DEFAULT 0,
    job_id              TEXT REFERENCES analysis_jobs(id)
);

CREATE INDEX IF NOT EXISTS idx_activities_job_id ON activities(job_id);
CREATE INDEX IF NOT EXISTS idx_activities_deleted_at ON activities(deleted_at);
CREATE INDEX IF NOT EXISTS idx_activities_started_at ON activities(started_at);
CREATE INDEX IF NOT EXISTS idx_analysis_jobs_status ON analysis_jobs(status);
CREATE INDEX IF NOT EXISTS idx_analysis_jobs_captured_at ON analysis_jobs(captured_at);
CREATE INDEX IF NOT EXISTS idx_api_usage_occurred_at ON api_usage(occurred_at);
CREATE INDEX IF NOT EXISTS idx_api_usage_job_id ON api_usage(job_id);
CREATE INDEX IF NOT EXISTS idx_reports_period_type ON reports(period_type, period_start);
