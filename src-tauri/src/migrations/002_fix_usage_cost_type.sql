CREATE TABLE api_usage_new (
    id                  TEXT PRIMARY KEY,
    occurred_at         TEXT NOT NULL,
    provider            TEXT NOT NULL,
    model               TEXT NOT NULL,
    input_tokens        INTEGER NOT NULL DEFAULT 0,
    output_tokens       INTEGER NOT NULL DEFAULT 0,
    estimated_cost_cents REAL NOT NULL DEFAULT 0,
    job_id              TEXT REFERENCES analysis_jobs(id)
);

INSERT INTO api_usage_new (id, occurred_at, provider, model, input_tokens, output_tokens, estimated_cost_cents, job_id)
    SELECT id, occurred_at, provider, model, input_tokens, output_tokens, estimated_cost_cents, job_id FROM api_usage;

DROP TABLE api_usage;
ALTER TABLE api_usage_new RENAME TO api_usage;

CREATE INDEX IF NOT EXISTS idx_api_usage_occurred_at ON api_usage(occurred_at);
CREATE INDEX IF NOT EXISTS idx_api_usage_job_id ON api_usage(job_id);
