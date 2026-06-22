# Daytrace MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use compose:subagent (recommended) or compose:execute to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Linux desktop app that captures work screenshots on a timer, analyzes them with vision AI, and generates daily/weekly/monthly reports — all local, privacy-first.

**Architecture:** Tauri 2 shell with Rust backend handling capture, AI, and storage; React 19 frontend for timeline and reports. SQLite for persistence, Linux Secret Service for API keys. Capture abstracted behind `CaptureProvider` trait for X11/Wayland portability.

**Tech Stack:** Tauri 2, Rust (tokio, rusqlite, reqwest, ashpd, xcap, image, chrono), React 19, TypeScript, Vite, Zustand, TanStack Query

---

## File Structure

```
src-tauri/
  Cargo.toml
  tauri.conf.json
  build.rs
  src/
    main.rs                     # Tauri bootstrap, plugins, managed state
    lib.rs                      # Re-export modules
    commands/
      mod.rs                    # All Tauri command handlers
    domain/
      mod.rs                    # Re-exports
      activity.rs               # Activity struct, Category enum
      report.rs                 # Report struct, PeriodType enum
      job.rs                    # AnalysisJob struct, JobStatus enum
      provider.rs               # VisionProvider, TextProvider traits
      capture.rs                # CaptureProvider trait, CapturedFrame
      settings.rs               # AppSettings struct
    capture/
      mod.rs                    # Provider selection, session management
      fake.rs                   # FakeCaptureProvider for testing
      x11.rs                    # X11CaptureProvider via xcap
      wayland.rs                # WaylandCaptureProvider via ashpd + PipeWire
    pipeline/
      mod.rs                    # Pipeline orchestration
      scheduler.rs              # CaptureScheduler (tokio interval)
      dedup.rs                  # Perceptual hash dedup (dHash)
      queue.rs                  # AnalysisJob queue worker
      retry.rs                  # Exponential backoff retry logic
    providers/
      mod.rs                    # Provider registry
      openai.rs                 # OpenAI vision + text
      anthropic.rs              # Anthropic vision + text
      validation.rs             # JSON schema validation, truncation
    storage/
      mod.rs                    # Database connection pool
      migrations.rs             # Schema migrations (numbered SQL files)
      activity_repo.rs          # Activity CRUD
      job_repo.rs               # AnalysisJob CRUD
      report_repo.rs            # Report CRUD
      settings_repo.rs          # Settings read/write
      usage_repo.rs             # API usage tracking
    reporting/
      mod.rs                    # Report generation orchestration
      aggregation.rs            # Deterministic local aggregation
      templates.rs              # Template loading (standard, concise, technical, okr)
      export.rs                 # Markdown file export
    platform/
      mod.rs                    # Platform detection (X11/Wayland)
      idle.rs                   # Idle detection via D-Bus
      tray.rs                   # System tray menu
      notifications.rs          # Desktop notifications via notify-rust
      paths.rs                  # App dirs, temp dir management
    migrations/
      001_initial.sql           # All tables
  tests/
    integration/
      pipeline_test.rs
      provider_test.rs
      storage_test.rs

src/
  main.tsx                      # React entry
  App.tsx                       # Router + layout
  pages/
    Today.tsx                   # Timeline + stats
    Reports.tsx                 # Report generation + history
    Settings.tsx                # All settings
    Onboarding.tsx              # First-run wizard
  components/
    Timeline.tsx                # Activity card list
    ActivityCard.tsx            # Single activity display
    ActivityEditor.tsx          # Edit modal
    ReportEditor.tsx            # Markdown editor + preview
    ReportPreview.tsx           # Rendered report
    CaptureToggle.tsx           # Start/pause button
    StatusBadge.tsx             # Current state indicator
    BudgetIndicator.tsx         # Daily spend display
  api/
    tauri.ts                    # Typed invoke wrappers + event listeners
  stores/
    recording.ts                # Recording state (Zustand)
    settings.ts                 # Settings cache (Zustand)
  lib/
    types.ts                    # Shared TypeScript types
    constants.ts                # Category list, default settings

index.html
package.json
vite.config.ts
tsconfig.json
```

---

## Phase 1: Project Scaffolding

### Task 1: Initialize Tauri 2 + React + Vite Project

**Covers:** §3 (tech stack), §9 (directory structure)

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.tsx`, `src/App.tsx`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Scaffold Tauri 2 + React project**

Run:
```bash
cd /home/somnus/proj/somReport
npm create tauri-app@latest . -- --template react-ts --manager npm
```

If the directory is not empty, use the interactive prompts to select:
- Frontend language: TypeScript
- Package manager: npm
- Template: react-ts

- [ ] **Step 2: Verify project builds**

Run:
```bash
cd /home/somnus/proj/somReport
npm install
npm run build
cd src-tauri && cargo build
```

Expected: Both frontend and Rust backend compile without errors.

- [ ] **Step 3: Configure Tauri window and app metadata**

Update `src-tauri/tauri.conf.json` to set the window size, title, and disable the default menu:
```json
{
  "productName": "daytrace",
  "version": "0.1.0",
  "identifier": "com.daytrace.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "Daytrace",
        "width": 1024,
        "height": 768,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    },
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.png"
    ]
  }
}
```

- [ ] **Step 4: Install frontend dependencies**

Run:
```bash
cd /home/somnus/proj/somReport
npm install zustand @tanstack/react-query react-router-dom
npm install -D @types/react @types/react-dom
```

- [ ] **Step 5: Set up basic React router**

Replace `src/App.tsx`:
```tsx
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const queryClient = new QueryClient()

function Today() {
  return <div>Today - Timeline</div>
}

function Reports() {
  return <div>Reports</div>
}

function Settings() {
  return <div>Settings</div>
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="flex h-screen">
          <nav className="w-48 bg-gray-100 p-4">
            <a href="/" className="block py-2">Today</a>
            <a href="/reports" className="block py-2">Reports</a>
            <a href="/settings" className="block py-2">Settings</a>
          </nav>
          <main className="flex-1 p-4 overflow-auto">
            <Routes>
              <Route path="/" element={<Today />} />
              <Route path="/reports" element={<Reports />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
    </QueryClientProvider>
  )
}
```

- [ ] **Step 6: Verify frontend builds**

Run:
```bash
npm run build
```

Expected: Vite builds successfully with no TypeScript errors.

- [ ] **Step 7: Commit**

```bash
git init
git add -A
git commit -m "chore: initialize Tauri 2 + React + Vite project"
```

---

### Task 2: Add Rust Dependencies

**Covers:** §3 (tech stack)

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add all required dependencies to Cargo.toml**

Replace `src-tauri/Cargo.toml` with:
```toml
[package]
name = "daytrace"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-autostart = "2"

serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
rusqlite = { version = "0.32", features = ["bundled"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
image = "0.25"
image_hasher = "3"
keyring = "3"
notify-rust = "4"
zbus = "5"
log = "0.4"
env_logger = "0.11"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 2: Verify Rust dependencies resolve**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Dependencies download and compile. Some warnings about unused imports are OK.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: add Rust dependencies for Daytrace"
```

---

### Task 3: SQLite Schema and Migrations

**Covers:** §7 (data model)

**Files:**
- Create: `src-tauri/src/migrations/001_initial.sql`
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/storage/migrations.rs`
- Create: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the migration SQL**

Create `src-tauri/src/migrations/001_initial.sql`:
```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS capture_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    started_at TEXT NOT NULL,
    stopped_at TEXT,
    backend TEXT NOT NULL,
    display_config_json TEXT
);

CREATE TABLE IF NOT EXISTS analysis_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    captured_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    image_hash TEXT,
    provider TEXT,
    model TEXT,
    created_at TEXT NOT NULL,
    finished_at TEXT
);

CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,
    category TEXT NOT NULL,
    summary TEXT NOT NULL,
    detail TEXT,
    confidence REAL NOT NULL,
    is_work_related INTEGER NOT NULL DEFAULT 1,
    source TEXT NOT NULL DEFAULT 'auto',
    edited_at TEXT,
    deleted_at TEXT,
    FOREIGN KEY (job_id) REFERENCES analysis_jobs(id)
);

CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY NOT NULL,
    period_type TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    template_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    model TEXT,
    prompt_version TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS api_usage (
    id TEXT PRIMARY KEY NOT NULL,
    occurred_at TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    estimated_cost_cents INTEGER NOT NULL DEFAULT 0,
    job_id TEXT,
    FOREIGN KEY (job_id) REFERENCES analysis_jobs(id)
);

CREATE INDEX IF NOT EXISTS idx_activities_started_at ON activities(started_at);
CREATE INDEX IF NOT EXISTS idx_activities_deleted_at ON activities(deleted_at);
CREATE INDEX IF NOT EXISTS idx_analysis_jobs_status ON analysis_jobs(status);
CREATE INDEX IF NOT EXISTS idx_reports_period ON reports(period_type, period_start);
CREATE INDEX IF NOT EXISTS idx_api_usage_occurred_at ON api_usage(occurred_at);
```

- [ ] **Step 2: Write the migrations module**

Create `src-tauri/src/storage/migrations.rs`:
```rust
use rusqlite::Connection;
use log::info;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001", include_str!("../migrations/001_initial.sql")),
];

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );"
    )?;

    let applied: Vec<String> = {
        let mut stmt = conn.prepare("SELECT name FROM _migrations ORDER BY name")?;
        stmt.query_map([], |row| row.get(0))?.collect::<Result<Vec<_>, _>>()?
    };

    for (name, sql) in MIGRATIONS {
        if applied.contains(&name.to_string()) {
            continue;
        }
        info!("Applying migration: {}", name);
        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO _migrations (name, applied_at) VALUES (?1, ?2)",
            rusqlite::params![name, chrono::Utc::now().to_rfc3339()],
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
            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_%' ORDER BY name"
            ).unwrap();
            stmt.query_map([], |row| row.get(0)).unwrap().collect::<Result<Vec<_>, _>>().unwrap()
        };

        assert_eq!(tables, vec![
            "activities", "analysis_jobs", "api_usage", "capture_sessions", "reports", "settings"
        ]);
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();
    }
}
```

- [ ] **Step 3: Write the storage module**

Create `src-tauri/src/storage/mod.rs`:
```rust
pub mod migrations;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &PathBuf) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")?;

        migrations::run_migrations(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        migrations::run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("database lock poisoned")
    }
}
```

- [ ] **Step 4: Set up module structure in lib.rs**

Create `src-tauri/src/lib.rs`:
```rust
pub mod storage;
```

Update `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run Rust tests**

Run:
```bash
cd src-tauri && cargo test
```

Expected: 2 tests pass (migrations_run_cleanly, migrations_are_idempotent).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/migrations/ src-tauri/src/storage/ src-tauri/src/lib.rs src-tauri/src/main.rs
git commit -m "feat: add SQLite schema and migration system"
```

---

## Phase 2: Domain Types and State Machine

### Task 4: Domain Types

**Covers:** §6 (AI contract), §7 (data model)

**Files:**
- Create: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/domain/activity.rs`
- Create: `src-tauri/src/domain/job.rs`
- Create: `src-tauri/src/domain/report.rs`
- Create: `src-tauri/src/domain/settings.rs`
- Create: `src-tauri/src/domain/capture.rs`
- Create: `src-tauri/src/domain/provider.rs`

- [ ] **Step 1: Write domain types**

Create `src-tauri/src/domain/mod.rs`:
```rust
pub mod activity;
pub mod job;
pub mod report;
pub mod settings;
pub mod capture;
pub mod provider;

pub use activity::{Activity, Category};
pub use job::{AnalysisJob, JobStatus};
pub use report::{Report, PeriodType};
pub use settings::AppSettings;
pub use capture::{CaptureProvider, CapturedFrame};
pub use provider::{VisionProvider, TextProvider, VisionResult, VisionItem};
```

Create `src-tauri/src/domain/activity.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub job_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: chrono::DateTime<chrono::Utc>,
    pub category: Category,
    pub summary: String,
    pub detail: Option<String>,
    pub confidence: f64,
    pub is_work_related: bool,
    pub source: String,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

Create `src-tauri/src/domain/job.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "processing" => Self::Processing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub id: String,
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub status: JobStatus,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub image_hash: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

Create `src-tauri/src/domain/report.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PeriodType {
    Daily,
    Weekly,
    Monthly,
}

impl PeriodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "weekly" => Self::Weekly,
            "monthly" => Self::Monthly,
            _ => Self::Daily,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub period_type: PeriodType,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub template_id: String,
    pub title: String,
    pub content_markdown: String,
    pub model: Option<String>,
    pub prompt_version: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

Create `src-tauri/src/domain/settings.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub capture_interval_sec: u32,
    pub idle_threshold_sec: u32,
    pub selected_displays: Vec<String>,
    pub provider: ProviderConfig,
    pub daily_budget_cents: u32,
    pub privacy_mode: bool,
    pub hash_similarity_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub vision_provider: String,
    pub vision_model: String,
    pub text_provider: String,
    pub text_model: String,
    pub base_url_overrides: std::collections::HashMap<String, String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            capture_interval_sec: 300,
            idle_threshold_sec: 300,
            selected_displays: vec![],
            provider: ProviderConfig {
                vision_provider: "openai".into(),
                vision_model: "gpt-4o".into(),
                text_provider: "openai".into(),
                text_model: "gpt-4o".into(),
                base_url_overrides: std::collections::HashMap::new(),
            },
            daily_budget_cents: 500,
            privacy_mode: false,
            hash_similarity_threshold: 0.9,
        }
    }
}
```

Create `src-tauri/src/domain/capture.rs`:
```rust
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub display_id: String,
    pub captured_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait CaptureProvider: Send + Sync {
    fn capabilities(&self) -> CaptureCapabilities;
    async fn start_session(&mut self) -> Result<String, String>;
    async fn capture_frame(&mut self) -> Result<CapturedFrame, String>;
    async fn stop_session(&mut self) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct CaptureCapabilities {
    pub supports_multi_display: bool,
    pub requires_user_consent: bool,
    pub backend_name: String,
}
```

Create `src-tauri/src/domain/provider.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionItem {
    pub category: String,
    pub summary: String,
    #[serde(default)]
    pub detail: Option<String>,
    pub confidence: f64,
    pub is_work_related: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResult {
    pub items: Vec<VisionItem>,
}

#[async_trait::async_trait]
pub trait VisionProvider: Send + Sync {
    async fn analyze_image(
        &self,
        image_data: &[u8],
        model: &str,
    ) -> Result<VisionResult, String>;
}

#[async_trait::async_trait]
pub trait TextProvider: Send + Sync {
    async fn generate_report(
        &self,
        activities_text: &str,
        template: &str,
        model: &str,
    ) -> Result<String, String>;
}
```

- [ ] **Step 2: Add async-trait dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
async-trait = "0.1"
```

- [ ] **Step 3: Update lib.rs**

Update `src-tauri/src/lib.rs`:
```rust
pub mod storage;
pub mod domain;
```

- [ ] **Step 4: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors. Warnings about unused items are OK.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/domain/ src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs
git commit -m "feat: add domain types for activities, jobs, reports, settings, capture"
```

---

### Task 5: Recording State Machine

**Covers:** §2 (start/pause), §5 (scheduler workflow)

**Files:**
- Create: `src-tauri/src/pipeline/mod.rs`
- Create: `src-tauri/src/pipeline/scheduler.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write recording state tests**

Create `src-tauri/src/pipeline/mod.rs`:
```rust
pub mod scheduler;
pub mod dedup;
pub mod queue;
pub mod retry;
```

Create `src-tauri/src/pipeline/scheduler.rs`:
```rust
use std::sync::Arc;
use tokio::sync::watch;
use log::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Stopped,
    Recording,
    Paused,
}

pub struct CaptureScheduler {
    state_tx: watch::Sender<RecordingState>,
    state_rx: watch::Receiver<RecordingState>,
    interval_sec: u32,
    idle_threshold_sec: u32,
}

impl CaptureScheduler {
    pub fn new(interval_sec: u32, idle_threshold_sec: u32) -> Self {
        let (state_tx, state_rx) = watch::channel(RecordingState::Stopped);
        Self {
            state_tx,
            state_rx,
            interval_sec,
            idle_threshold_sec,
        }
    }

    pub fn state(&self) -> RecordingState {
        self.state_rx.borrow().clone()
    }

    pub fn state_rx(&self) -> watch::Receiver<RecordingState> {
        self.state_rx.clone()
    }

    pub fn start(&self) -> Result<(), String> {
        let current = self.state_rx.borrow().clone();
        match current {
            RecordingState::Stopped | RecordingState::Paused => {
                self.state_tx.send(RecordingState::Recording)
                    .map_err(|_| "failed to send state".to_string())?;
                info!("Recording started");
                Ok(())
            }
            RecordingState::Recording => {
                Err("already recording".to_string())
            }
        }
    }

    pub fn pause(&self) -> Result<(), String> {
        let current = self.state_rx.borrow().clone();
        match current {
            RecordingState::Recording => {
                self.state_tx.send(RecordingState::Paused)
                    .map_err(|_| "failed to send state".to_string())?;
                info!("Recording paused");
                Ok(())
            }
            _ => Err("not recording".to_string()),
        }
    }

    pub fn stop(&self) -> Result<(), String> {
        let current = self.state_rx.borrow().clone();
        match current {
            RecordingState::Recording | RecordingState::Paused => {
                self.state_tx.send(RecordingState::Stopped)
                    .map_err(|_| "failed to send state".to_string())?;
                info!("Recording stopped");
                Ok(())
            }
            RecordingState::Stopped => {
                Err("already stopped".to_string())
            }
        }
    }

    pub fn interval_sec(&self) -> u32 {
        self.interval_sec
    }

    pub fn idle_threshold_sec(&self) -> u32 {
        self.idle_threshold_sec
    }

    pub fn set_interval(&mut self, sec: u32) {
        self.interval_sec = sec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_stopped() {
        let scheduler = CaptureScheduler::new(300, 300);
        assert_eq!(scheduler.state(), RecordingState::Stopped);
    }

    #[test]
    fn test_start_from_stopped() {
        let scheduler = CaptureScheduler::new(300, 300);
        scheduler.start().unwrap();
        assert_eq!(scheduler.state(), RecordingState::Recording);
    }

    #[test]
    fn test_start_from_paused() {
        let scheduler = CaptureScheduler::new(300, 300);
        scheduler.start().unwrap();
        scheduler.pause().unwrap();
        scheduler.start().unwrap();
        assert_eq!(scheduler.state(), RecordingState::Recording);
    }

    #[test]
    fn test_start_from_recording_fails() {
        let scheduler = CaptureScheduler::new(300, 300);
        scheduler.start().unwrap();
        assert!(scheduler.start().is_err());
    }

    #[test]
    fn test_pause_from_recording() {
        let scheduler = CaptureScheduler::new(300, 300);
        scheduler.start().unwrap();
        scheduler.pause().unwrap();
        assert_eq!(scheduler.state(), RecordingState::Paused);
    }

    #[test]
    fn test_pause_from_stopped_fails() {
        let scheduler = CaptureScheduler::new(300, 300);
        assert!(scheduler.pause().is_err());
    }

    #[test]
    fn test_stop_from_recording() {
        let scheduler = CaptureScheduler::new(300, 300);
        scheduler.start().unwrap();
        scheduler.stop().unwrap();
        assert_eq!(scheduler.state(), RecordingState::Stopped);
    }

    #[test]
    fn test_stop_from_paused() {
        let scheduler = CaptureScheduler::new(300, 300);
        scheduler.start().unwrap();
        scheduler.pause().unwrap();
        scheduler.stop().unwrap();
        assert_eq!(scheduler.state(), RecordingState::Stopped);
    }

    #[test]
    fn test_stop_from_stopped_fails() {
        let scheduler = CaptureScheduler::new(300, 300);
        assert!(scheduler.stop().is_err());
    }

    #[test]
    fn test_state_rx_receives_updates() {
        let scheduler = CaptureScheduler::new(300, 300);
        let mut rx = scheduler.state_rx();

        assert_eq!(*rx.borrow_and_update(), RecordingState::Stopped);

        scheduler.start().unwrap();
        assert_eq!(*rx.borrow_and_update(), RecordingState::Recording);

        scheduler.pause().unwrap();
        assert_eq!(*rx.borrow_and_update(), RecordingState::Paused);
    }
}
```

- [ ] **Step 2: Run scheduler tests**

Run:
```bash
cd src-tauri && cargo test --lib pipeline::scheduler
```

Expected: 10 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pipeline/
git commit -m "feat: add recording state machine with start/pause/stop"
```

---

### Task 6: Fake Capture Provider

**Covers:** §4 (CaptureProvider trait), §11 (fake capture provider)

**Files:**
- Create: `src-tauri/src/capture/mod.rs`
- Create: `src-tauri/src/capture/fake.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write fake capture provider**

Create `src-tauri/src/capture/mod.rs`:
```rust
pub mod fake;

pub use fake::FakeCaptureProvider;
```

Create `src-tauri/src/capture/fake.rs`:
```rust
use crate::domain::capture::{CaptureCapabilities, CaptureProvider, CapturedFrame};
use async_trait::async_trait;

pub struct FakeCaptureProvider {
    frame_count: u64,
    session_active: bool,
}

impl FakeCaptureProvider {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            session_active: false,
        }
    }
}

#[async_trait]
impl CaptureProvider for FakeCaptureProvider {
    fn capabilities(&self) -> CaptureCapabilities {
        CaptureCapabilities {
            supports_multi_display: false,
            requires_user_consent: false,
            backend_name: "fake".to_string(),
        }
    }

    async fn start_session(&mut self) -> Result<String, String> {
        self.session_active = true;
        self.frame_count = 0;
        Ok("fake-session".to_string())
    }

    async fn capture_frame(&mut self) -> Result<CapturedFrame, String> {
        if !self.session_active {
            return Err("no active session".to_string());
        }

        self.frame_count += 1;

        // Generate a minimal valid JPEG (1x1 white pixel)
        let jpeg_data = vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
            0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
            0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
            0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
            0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
            0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
            0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
            0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
            0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00,
            0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0A, 0x0B, 0xFF, 0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03,
            0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D,
            0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06,
            0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08,
            0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72,
            0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
            0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45,
            0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
            0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75,
            0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
            0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3,
            0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6,
            0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9,
            0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
            0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4,
            0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01,
            0x00, 0x00, 0x3F, 0x00, 0x7B, 0x40, 0x1B, 0xFF, 0xD9,
        ];

        Ok(CapturedFrame {
            data: jpeg_data,
            width: 1,
            height: 1,
            display_id: "fake-display-0".to_string(),
            captured_at: chrono::Utc::now(),
        })
    }

    async fn stop_session(&mut self) -> Result<(), String> {
        self.session_active = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_provider_lifecycle() {
        let mut provider = FakeCaptureProvider::new();

        let caps = provider.capabilities();
        assert_eq!(caps.backend_name, "fake");
        assert!(!caps.requires_user_consent);

        let session_id = provider.start_session().await.unwrap();
        assert_eq!(session_id, "fake-session");

        let frame = provider.capture_frame().await.unwrap();
        assert_eq!(frame.display_id, "fake-display-0");
        assert!(!frame.data.is_empty());

        provider.stop_session().await.unwrap();
    }

    #[tokio::test]
    async fn test_fake_provider_capture_without_session_fails() {
        let mut provider = FakeCaptureProvider::new();
        assert!(provider.capture_frame().await.is_err());
    }
}
```

- [ ] **Step 2: Update lib.rs**

Update `src-tauri/src/lib.rs`:
```rust
pub mod storage;
pub mod domain;
pub mod capture;
pub mod pipeline;
```

- [ ] **Step 3: Run tests**

Run:
```bash
cd src-tauri && cargo test
```

Expected: All tests pass (12+ tests).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/capture/ src-tauri/src/lib.rs
git commit -m "feat: add fake capture provider for testing"
```

---

### Task 7: Perceptual Hash Deduplication

**Covers:** §5 (dHash dedup)

**Files:**
- Create: `src-tauri/src/pipeline/dedup.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Write dedup module with tests**

Create `src-tauri/src/pipeline/dedup.rs`:
```rust
use image::GenericImageView;
use log::debug;

pub struct DedupChecker {
    threshold: f64,
    last_hash: Option<u64>,
}

impl DedupChecker {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            last_hash: None,
        }
    }

    pub fn check_and_update(&mut self, image_data: &[u8]) -> Result<bool, String> {
        let hash = compute_dhash(image_data)?;

        if let Some(ref last) = self.last_hash {
            let similarity = hash_similarity(*last, hash);
            debug!("Hash similarity: {:.3} (threshold: {:.3})", similarity, self.threshold);
            if similarity >= self.threshold {
                return Ok(true); // duplicate
            }
        }

        self.last_hash = Some(hash);
        Ok(false) // new frame
    }

    pub fn reset(&mut self) {
        self.last_hash = None;
    }
}

fn compute_dhash(image_data: &[u8]) -> Result<u64, String> {
    let img = image::load_from_memory(image_data)
        .map_err(|e| format!("failed to load image: {}", e))?;

    let gray = img.resize_exact(9, 8, image::imageops::FilterType::Triangle)
        .to_luma8();

    let pixels: Vec<u8> = gray.into_raw();
    let mut hash: u64 = 0;

    for row in 0..8 {
        for col in 0..8 {
            let left = pixels[row * 9 + col];
            let right = pixels[row * 9 + col + 1];
            if left < right {
                hash |= 1 << (row * 8 + col);
            }
        }
    }

    Ok(hash)
}

fn hash_similarity(a: u64, b: u64) -> f64 {
    let xor = a ^ b;
    let bits_different = xor.count_ones();
    1.0 - (bits_different as f64 / 64.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_jpeg() -> Vec<u8> {
        // Create a simple 100x100 red image in memory
        let img = image::RgbImage::from_fn(100, 100, |_, _| image::Rgb([255, 0, 0]));
        let mut buf = Vec::new();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 75);
        img.write_with_encoder(encoder).unwrap();
        buf
    }

    #[test]
    fn test_dhash_deterministic() {
        let data = make_test_jpeg();
        let h1 = compute_dhash(&data).unwrap();
        let h2 = compute_dhash(&data).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_identical_frames_are_duplicates() {
        let data = make_test_jpeg();
        let mut checker = DedupChecker::new(0.9);
        assert!(!checker.check_and_update(&data).unwrap()); // first frame
        assert!(checker.check_and_update(&data).unwrap()); // duplicate
    }

    #[test]
    fn test_different_frames_not_duplicates() {
        let red = {
            let img = image::RgbImage::from_fn(100, 100, |_, _| image::Rgb([255, 0, 0]));
            let mut buf = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 75);
            img.write_with_encoder(encoder).unwrap();
            buf
        };
        let blue = {
            let img = image::RgbImage::from_fn(100, 100, |_, _| image::Rgb([0, 0, 255]));
            let mut buf = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 75);
            img.write_with_encoder(encoder).unwrap();
            buf
        };

        let mut checker = DedupChecker::new(0.9);
        assert!(!checker.check_and_update(&red).unwrap());
        assert!(!checker.check_and_update(&blue).unwrap());
    }

    #[test]
    fn test_reset_clears_history() {
        let data = make_test_jpeg();
        let mut checker = DedupChecker::new(0.9);
        checker.check_and_update(&data).unwrap();
        checker.reset();
        assert!(!checker.check_and_update(&data).unwrap()); // not a duplicate after reset
    }

    #[test]
    fn test_hash_similarity_identical() {
        assert_eq!(hash_similarity(0xFFFF, 0xFFFF), 1.0);
    }

    #[test]
    fn test_hash_similarity_completely_different() {
        assert_eq!(hash_similarity(0x0000, 0xFFFF), 0.0);
    }
}
```

- [ ] **Step 2: Run dedup tests**

Run:
```bash
cd src-tauri && cargo test --lib pipeline::dedup
```

Expected: 6 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pipeline/dedup.rs
git commit -m "feat: add perceptual hash deduplication (dHash)"
```

---

## Phase 3: Storage Repositories

### Task 8: Activity Repository

**Covers:** §7 (data model), §5 (activity storage)

**Files:**
- Create: `src-tauri/src/storage/activity_repo.rs`
- Modify: `src-tauri/src/storage/mod.rs`

- [ ] **Step 1: Write activity repository with tests**

Create `src-tauri/src/storage/activity_repo.rs`:
```rust
use crate::domain::{Activity, Category};
use crate::storage::Database;
use chrono::{DateTime, Utc, NaiveDate};
use log::warn;

impl Database {
    pub fn insert_activity(&self, activity: &Activity) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO activities (id, job_id, started_at, ended_at, category, summary, detail, confidence, is_work_related, source, edited_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                activity.id,
                activity.job_id,
                activity.started_at.to_rfc3339(),
                activity.ended_at.to_rfc3339(),
                activity.category.as_str(),
                activity.summary,
                activity.detail,
                activity.confidence,
                activity.is_work_related as i32,
                activity.source,
                activity.edited_at.map(|t| t.to_rfc3339()),
                activity.deleted_at.map(|t| t.to_rfc3339()),
            ],
        ).map_err(|e| format!("insert activity: {}", e))?;
        Ok(())
    }

    pub fn get_activities_for_date(&self, date: NaiveDate) -> Result<Vec<Activity>, String> {
        let conn = self.conn();
        let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = (date + chrono::Duration::days(1)).and_hms_opt(0, 0, 0).unwrap().and_utc();

        let mut stmt = conn.prepare(
            "SELECT id, job_id, started_at, ended_at, category, summary, detail, confidence, is_work_related, source, edited_at, deleted_at
             FROM activities
             WHERE started_at >= ?1 AND started_at < ?2 AND deleted_at IS NULL
             ORDER BY started_at"
        ).map_err(|e| format!("prepare: {}", e))?;

        let rows = stmt.query_map(rusqlite::params![start.to_rfc3339(), end.to_rfc3339()], |row| {
            Ok(Activity {
                id: row.get(0)?,
                job_id: row.get(1)?,
                started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap().with_timezone(&Utc),
                ended_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap().with_timezone(&Utc),
                category: Category::from_str(&row.get::<_, String>(4)?),
                summary: row.get(5)?,
                detail: row.get(6)?,
                confidence: row.get(7)?,
                is_work_related: row.get::<_, i32>(8)? != 0,
                source: row.get(9)?,
                edited_at: row.get::<_, Option<String>>(10)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                deleted_at: row.get::<_, Option<String>>(11)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            })
        }).map_err(|e| format!("query: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("collect: {}", e))
    }

    pub fn update_activity(&self, activity: &Activity) -> Result<(), String> {
        let conn = self.conn();
        let affected = conn.execute(
            "UPDATE activities SET category = ?1, summary = ?2, detail = ?3, is_work_related = ?4, edited_at = ?5
             WHERE id = ?6 AND deleted_at IS NULL",
            rusqlite::params![
                activity.category.as_str(),
                activity.summary,
                activity.detail,
                activity.is_work_related as i32,
                Utc::now().to_rfc3339(),
                activity.id,
            ],
        ).map_err(|e| format!("update activity: {}", e))?;

        if affected == 0 {
            return Err("activity not found or already deleted".to_string());
        }
        Ok(())
    }

    pub fn soft_delete_activity(&self, id: &str) -> Result<(), String> {
        let conn = self.conn();
        let affected = conn.execute(
            "UPDATE activities SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            rusqlite::params![Utc::now().to_rfc3339(), id],
        ).map_err(|e| format!("delete activity: {}", e))?;

        if affected == 0 {
            return Err("activity not found or already deleted".to_string());
        }
        Ok(())
    }

    pub fn get_activities_in_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<Activity>, String> {
        let conn = self.conn();
        let start_dt = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_dt = (end + chrono::Duration::days(1)).and_hms_opt(0, 0, 0).unwrap().and_utc();

        let mut stmt = conn.prepare(
            "SELECT id, job_id, started_at, ended_at, category, summary, detail, confidence, is_work_related, source, edited_at, deleted_at
             FROM activities
             WHERE started_at >= ?1 AND started_at < ?2 AND deleted_at IS NULL
             ORDER BY started_at"
        ).map_err(|e| format!("prepare: {}", e))?;

        let rows = stmt.query_map(rusqlite::params![start_dt.to_rfc3339(), end_dt.to_rfc3339()], |row| {
            Ok(Activity {
                id: row.get(0)?,
                job_id: row.get(1)?,
                started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap().with_timezone(&Utc),
                ended_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap().with_timezone(&Utc),
                category: Category::from_str(&row.get::<_, String>(4)?),
                summary: row.get(5)?,
                detail: row.get(6)?,
                confidence: row.get(7)?,
                is_work_related: row.get::<_, i32>(8)? != 0,
                source: row.get(9)?,
                edited_at: row.get::<_, Option<String>>(10)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                deleted_at: row.get::<_, Option<String>>(11)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            })
        }).map_err(|e| format!("query: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("collect: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Activity;

    fn make_activity(id: &str, summary: &str, hour: u32) -> Activity {
        let base = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        Activity {
            id: id.to_string(),
            job_id: "job-1".to_string(),
            started_at: base.and_hms_opt(hour, 0, 0).unwrap().and_utc(),
            ended_at: base.and_hms_opt(hour, 5, 0).unwrap().and_utc(),
            category: Category::Development,
            summary: summary.to_string(),
            detail: None,
            confidence: 0.9,
            is_work_related: true,
            source: "auto".to_string(),
            edited_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let db = Database::new_in_memory().unwrap();
        let activity = make_activity("a1", "Coding", 10);
        db.insert_activity(&activity).unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        let activities = db.get_activities_for_date(date).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].summary, "Coding");
    }

    #[test]
    fn test_soft_delete() {
        let db = Database::new_in_memory().unwrap();
        let activity = make_activity("a1", "Coding", 10);
        db.insert_activity(&activity).unwrap();
        db.soft_delete_activity("a1").unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        let activities = db.get_activities_for_date(date).unwrap();
        assert_eq!(activities.len(), 0);
    }

    #[test]
    fn test_update() {
        let db = Database::new_in_memory().unwrap();
        let mut activity = make_activity("a1", "Coding", 10);
        db.insert_activity(&activity).unwrap();

        activity.summary = "Updated summary".to_string();
        activity.category = Category::Meeting;
        db.update_activity(&activity).unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        let activities = db.get_activities_for_date(date).unwrap();
        assert_eq!(activities[0].summary, "Updated summary");
        assert_eq!(activities[0].category, Category::Meeting);
    }

    #[test]
    fn test_get_activities_in_range() {
        let db = Database::new_in_memory().unwrap();
        db.insert_activity(&make_activity("a1", "Day 1", 10)).unwrap();
        db.insert_activity(&make_activity("a2", "Day 2", 14)).unwrap();

        // Both are on same day in make_activity, so range of 1 day returns both
        let start = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 23).unwrap();
        let activities = db.get_activities_in_range(start, end).unwrap();
        assert_eq!(activities.len(), 2);
    }
}
```

- [ ] **Step 2: Update storage mod.rs**

Update `src-tauri/src/storage/mod.rs`:
```rust
pub mod migrations;
pub mod activity_repo;

// ... rest of file unchanged
```

- [ ] **Step 3: Run tests**

Run:
```bash
cd src-tauri && cargo test --lib storage::activity_repo
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/storage/activity_repo.rs src-tauri/src/storage/mod.rs
git commit -m "feat: add activity repository with CRUD operations"
```

---

### Task 9: Job Repository

**Covers:** §7 (data model), §5 (job queue)

**Files:**
- Create: `src-tauri/src/storage/job_repo.rs`
- Modify: `src-tauri/src/storage/mod.rs`

- [ ] **Step 1: Write job repository**

Create `src-tauri/src/storage/job_repo.rs`:
```rust
use crate::domain::{AnalysisJob, JobStatus};
use crate::storage::Database;
use chrono::{DateTime, Utc};

impl Database {
    pub fn insert_job(&self, job: &AnalysisJob) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO analysis_jobs (id, captured_at, status, attempts, last_error, image_hash, provider, model, created_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                job.id,
                job.captured_at.to_rfc3339(),
                job.status.as_str(),
                job.attempts,
                job.last_error,
                job.image_hash,
                job.provider,
                job.model,
                job.created_at.to_rfc3339(),
                job.finished_at.map(|t| t.to_rfc3339()),
            ],
        ).map_err(|e| format!("insert job: {}", e))?;
        Ok(())
    }

    pub fn claim_next_pending_job(&self) -> Result<Option<AnalysisJob>, String> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, captured_at, status, attempts, last_error, image_hash, provider, model, created_at, finished_at
             FROM analysis_jobs
             WHERE status = 'pending' AND attempts < 3
             ORDER BY created_at ASC
             LIMIT 1"
        ).map_err(|e| format!("prepare: {}", e))?;

        let mut rows = stmt.query_map([], |row| {
            Ok(AnalysisJob {
                id: row.get(0)?,
                captured_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap().with_timezone(&Utc),
                status: JobStatus::from_str(&row.get::<_, String>(2)?),
                attempts: row.get(3)?,
                last_error: row.get(4)?,
                image_hash: row.get(5)?,
                provider: row.get(6)?,
                model: row.get(7)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap().with_timezone(&Utc),
                finished_at: row.get::<_, Option<String>>(9)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            })
        }).map_err(|e| format!("query: {}", e))?;

        match rows.next() {
            Some(Ok(job)) => {
                conn.execute(
                    "UPDATE analysis_jobs SET status = 'processing', attempts = attempts + 1 WHERE id = ?1",
                    rusqlite::params![job.id],
                ).map_err(|e| format!("claim job: {}", e))?;
                Ok(Some(job))
            }
            Some(Err(e)) => Err(format!("row error: {}", e)),
            None => Ok(None),
        }
    }

    pub fn complete_job(&self, id: &str, provider: &str, model: &str) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "UPDATE analysis_jobs SET status = 'completed', provider = ?1, model = ?2, finished_at = ?3 WHERE id = ?4",
            rusqlite::params![provider, model, Utc::now().to_rfc3339(), id],
        ).map_err(|e| format!("complete job: {}", e))?;
        Ok(())
    }

    pub fn fail_job(&self, id: &str, error: &str) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "UPDATE analysis_jobs SET status = 'failed', last_error = ?1, finished_at = ?2 WHERE id = ?3",
            rusqlite::params![error, Utc::now().to_rfc3339(), id],
        ).map_err(|e| format!("fail job: {}", e))?;
        Ok(())
    }

    pub fn retry_job(&self, id: &str) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "UPDATE analysis_jobs SET status = 'pending' WHERE id = ?1 AND attempts < 3",
            rusqlite::params![id],
        ).map_err(|e| format!("retry job: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_job(id: &str) -> AnalysisJob {
        AnalysisJob {
            id: id.to_string(),
            captured_at: Utc::now(),
            status: JobStatus::Pending,
            attempts: 0,
            last_error: None,
            image_hash: Some("abc123".to_string()),
            provider: None,
            model: None,
            created_at: Utc::now(),
            finished_at: None,
        }
    }

    #[test]
    fn test_insert_and_claim() {
        let db = Database::new_in_memory().unwrap();
        let job = make_job("j1");
        db.insert_job(&job).unwrap();

        let claimed = db.claim_next_pending_job().unwrap().unwrap();
        assert_eq!(claimed.id, "j1");
    }

    #[test]
    fn test_claim_sets_processing() {
        let db = Database::new_in_memory().unwrap();
        db.insert_job(&make_job("j1")).unwrap();

        db.claim_next_pending_job().unwrap();
        // Second claim should return None (no more pending)
        let result = db.claim_next_pending_job().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_complete_job() {
        let db = Database::new_in_memory().unwrap();
        db.insert_job(&make_job("j1")).unwrap();
        db.claim_next_pending_job().unwrap();
        db.complete_job("j1", "openai", "gpt-4o").unwrap();
    }

    #[test]
    fn test_fail_job() {
        let db = Database::new_in_memory().unwrap();
        db.insert_job(&make_job("j1")).unwrap();
        db.fail_job("j1", "network error").unwrap();
    }
}
```

- [ ] **Step 2: Update storage mod.rs**

Update `src-tauri/src/storage/mod.rs`:
```rust
pub mod migrations;
pub mod activity_repo;
pub mod job_repo;

// ... rest of file unchanged
```

- [ ] **Step 3: Run tests**

Run:
```bash
cd src-tauri && cargo test --lib storage::job_repo
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/storage/job_repo.rs src-tauri/src/storage/mod.rs
git commit -m "feat: add job repository with claim/complete/fail/retry"
```

---

### Task 10: Settings and Report Repositories

**Covers:** §7 (data model)

**Files:**
- Create: `src-tauri/src/storage/settings_repo.rs`
- Create: `src-tauri/src/storage/report_repo.rs`
- Create: `src-tauri/src/storage/usage_repo.rs`
- Modify: `src-tauri/src/storage/mod.rs`

- [ ] **Step 1: Write settings repository**

Create `src-tauri/src/storage/settings_repo.rs`:
```rust
use crate::domain::AppSettings;
use crate::storage::Database;

impl Database {
    pub fn get_settings(&self) -> Result<AppSettings, String> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")
            .map_err(|e| format!("prepare: {}", e))?;

        let rows: Vec<(String, String)> = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| format!("query: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("collect: {}", e))?;

        if rows.is_empty() {
            return Ok(AppSettings::default());
        }

        let map: std::collections::HashMap<String, String> = rows.into_iter().collect();

        Ok(AppSettings {
            capture_interval_sec: map.get("capture_interval_sec")
                .and_then(|v| v.parse().ok()).unwrap_or(300),
            idle_threshold_sec: map.get("idle_threshold_sec")
                .and_then(|v| v.parse().ok()).unwrap_or(300),
            selected_displays: map.get("selected_displays_json")
                .and_then(|v| serde_json::from_str(v).ok()).unwrap_or_default(),
            provider: map.get("provider_config_json")
                .and_then(|v| serde_json::from_str(v).ok())
                .unwrap_or_else(|| AppSettings::default().provider),
            daily_budget_cents: map.get("daily_budget_cents")
                .and_then(|v| v.parse().ok()).unwrap_or(500),
            privacy_mode: map.get("privacy_mode")
                .map(|v| v == "true").unwrap_or(false),
            hash_similarity_threshold: map.get("hash_similarity_threshold")
                .and_then(|v| v.parse().ok()).unwrap_or(0.9),
        })
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        let conn = self.conn();
        let pairs = vec![
            ("capture_interval_sec", settings.capture_interval_sec.to_string()),
            ("idle_threshold_sec", settings.idle_threshold_sec.to_string()),
            ("selected_displays_json", serde_json::to_string(&settings.selected_displays).unwrap_or_default()),
            ("provider_config_json", serde_json::to_string(&settings.provider).unwrap_or_default()),
            ("daily_budget_cents", settings.daily_budget_cents.to_string()),
            ("privacy_mode", settings.privacy_mode.to_string()),
            ("hash_similarity_threshold", settings.hash_similarity_threshold.to_string()),
        ];

        for (key, value) in pairs {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                rusqlite::params![key, value],
            ).map_err(|e| format!("save setting {}: {}", key, e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_when_empty() {
        let db = Database::new_in_memory().unwrap();
        let settings = db.get_settings().unwrap();
        assert_eq!(settings.capture_interval_sec, 300);
        assert_eq!(settings.daily_budget_cents, 500);
    }

    #[test]
    fn test_save_and_load_settings() {
        let db = Database::new_in_memory().unwrap();
        let mut settings = AppSettings::default();
        settings.capture_interval_sec = 60;
        settings.daily_budget_cents = 1000;
        db.save_settings(&settings).unwrap();

        let loaded = db.get_settings().unwrap();
        assert_eq!(loaded.capture_interval_sec, 60);
        assert_eq!(loaded.daily_budget_cents, 1000);
    }
}
```

- [ ] **Step 2: Write report repository**

Create `src-tauri/src/storage/report_repo.rs`:
```rust
use crate::domain::{Report, PeriodType};
use crate::storage::Database;
use chrono::{DateTime, NaiveDate, Utc};

impl Database {
    pub fn insert_report(&self, report: &Report) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO reports (id, period_type, period_start, period_end, template_id, title, content_markdown, model, prompt_version, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                report.id,
                report.period_type.as_str(),
                report.period_start.to_string(),
                report.period_end.to_string(),
                report.template_id,
                report.title,
                report.content_markdown,
                report.model,
                report.prompt_version,
                report.created_at.to_rfc3339(),
                report.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| format!("insert report: {}", e))?;
        Ok(())
    }

    pub fn get_reports(&self, period_type: Option<PeriodType>) -> Result<Vec<Report>, String> {
        let conn = self.conn();
        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &period_type {
            Some(pt) => (
                "SELECT id, period_type, period_start, period_end, template_id, title, content_markdown, model, prompt_version, created_at, updated_at
                 FROM reports WHERE period_type = ?1 ORDER BY period_start DESC",
                vec![Box::new(pt.as_str().to_string())],
            ),
            None => (
                "SELECT id, period_type, period_start, period_end, template_id, title, content_markdown, model, prompt_version, created_at, updated_at
                 FROM reports ORDER BY period_start DESC",
                vec![],
            ),
        };

        let mut stmt = conn.prepare(sql).map_err(|e| format!("prepare: {}", e))?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(Report {
                id: row.get(0)?,
                period_type: PeriodType::from_str(&row.get::<_, String>(1)?),
                period_start: NaiveDate::parse_from_str(&row.get::<_, String>(2)?, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                period_end: NaiveDate::parse_from_str(&row.get::<_, String>(3)?, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                template_id: row.get(4)?,
                title: row.get(5)?,
                content_markdown: row.get(6)?,
                model: row.get(7)?,
                prompt_version: row.get(8)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                    .unwrap().with_timezone(&Utc),
            })
        }).map_err(|e| format!("query: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| format!("collect: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get_report() {
        let db = Database::new_in_memory().unwrap();
        let now = Utc::now();
        let report = Report {
            id: "r1".to_string(),
            period_type: PeriodType::Daily,
            period_start: NaiveDate::from_ymd_opt(2026, 6, 23).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 6, 23).unwrap(),
            template_id: "standard".to_string(),
            title: "Daily Report".to_string(),
            content_markdown: "# Report".to_string(),
            model: Some("gpt-4o".to_string()),
            prompt_version: Some("1".to_string()),
            created_at: now,
            updated_at: now,
        };
        db.insert_report(&report).unwrap();

        let reports = db.get_reports(Some(PeriodType::Daily)).unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].title, "Daily Report");
    }
}
```

- [ ] **Step 3: Write usage repository**

Create `src-tauri/src/storage/usage_repo.rs`:
```rust
use crate::storage::Database;
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct UsageEntry {
    pub id: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub estimated_cost_cents: i64,
    pub job_id: Option<String>,
}

impl Database {
    pub fn record_usage(&self, entry: &UsageEntry) -> Result<(), String> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO api_usage (id, occurred_at, provider, model, input_tokens, output_tokens, estimated_cost_cents, job_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                entry.id,
                entry.occurred_at.to_rfc3339(),
                entry.provider,
                entry.model,
                entry.input_tokens,
                entry.output_tokens,
                entry.estimated_cost_cents,
                entry.job_id,
            ],
        ).map_err(|e| format!("record usage: {}", e))?;
        Ok(())
    }

    pub fn get_daily_usage_cents(&self, date: NaiveDate) -> Result<i64, String> {
        let conn = self.conn();
        let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = (date + chrono::Duration::days(1)).and_hms_opt(0, 0, 0).unwrap().and_utc();

        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(estimated_cost_cents), 0) FROM api_usage WHERE occurred_at >= ?1 AND occurred_at < ?2",
            rusqlite::params![start.to_rfc3339(), end.to_rfc3339()],
            |row| row.get(0),
        ).map_err(|e| format!("query usage: {}", e))?;

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query_usage() {
        let db = Database::new_in_memory().unwrap();
        let entry = UsageEntry {
            id: Uuid::new_v4().to_string(),
            occurred_at: Utc::now(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            estimated_cost_cents: 15,
            job_id: None,
        };
        db.record_usage(&entry).unwrap();

        let today = Utc::now().date_naive();
        let total = db.get_daily_usage_cents(today).unwrap();
        assert_eq!(total, 15);
    }
}
```

- [ ] **Step 4: Update storage mod.rs**

Update `src-tauri/src/storage/mod.rs`:
```rust
pub mod migrations;
pub mod activity_repo;
pub mod job_repo;
pub mod settings_repo;
pub mod report_repo;
pub mod usage_repo;

// ... rest of file unchanged
```

- [ ] **Step 5: Run all storage tests**

Run:
```bash
cd src-tauri && cargo test --lib storage
```

Expected: All storage tests pass (10+ tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/storage/
git commit -m "feat: add settings, report, and usage repositories"
```

---

## Phase 4: AI Provider Integration

### Task 11: JSON Validation for Vision Results

**Covers:** §6 (AI contract, JSON schema validation)

**Files:**
- Create: `src-tauri/src/providers/mod.rs`
- Create: `src-tauri/src/providers/validation.rs`

- [ ] **Step 1: Write validation module**

Create `src-tauri/src/providers/mod.rs`:
```rust
pub mod validation;
pub mod openai;
pub mod anthropic;
```

Create `src-tauri/src/providers/validation.rs`:
```rust
use crate::domain::{VisionResult, VisionItem, Category};
use log::warn;

pub fn validate_vision_result(raw_json: &str) -> Result<VisionResult, String> {
    let result: VisionResult = serde_json::from_str(raw_json)
        .map_err(|e| format!("invalid JSON structure: {}", e))?;

    if result.items.is_empty() {
        return Err("no items in vision result".to_string());
    }

    let mut validated_items = Vec::new();
    for (i, mut item) in result.items.into_iter().enumerate() {
        if item.summary.is_empty() {
            warn!("Item {} has empty summary, skipping", i);
            continue;
        }

        // Truncate summary to 80 chars
        if item.summary.len() > 80 {
            item.summary = truncate_to_char_boundary(&item.summary, 80);
        }

        // Truncate detail to 240 chars
        if let Some(ref detail) = item.detail {
            if detail.len() > 240 {
                item.detail = Some(truncate_to_char_boundary(detail, 240));
            }
        }

        // Clamp confidence to 0.0-1.0
        item.confidence = item.confidence.clamp(0.0, 1.0);

        // Normalize category - unknown values become "other"
        let _ = Category::from_str(&item.category); // just validates it parses

        validated_items.push(item);
    }

    if validated_items.is_empty() {
        return Err("all items were invalid after validation".to_string());
    }

    Ok(VisionResult { items: validated_items })
}

fn truncate_to_char_boundary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut boundary = max_len;
    while !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    s[..boundary].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let json = r#"{
            "items": [{
                "category": "development",
                "summary": "Writing Rust code in VS Code",
                "detail": null,
                "confidence": 0.9,
                "is_work_related": true
            }]
        }"#;
        let result = validate_vision_result(json).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].category, "development");
    }

    #[test]
    fn test_invalid_json_fails() {
        assert!(validate_vision_result("not json").is_err());
    }

    #[test]
    fn test_empty_items_fails() {
        assert!(validate_vision_result(r#"{"items": []}"#).is_err());
    }

    #[test]
    fn test_truncates_long_summary() {
        let long_summary = "A".repeat(200);
        let json = format!(r#"{{
            "items": [{{
                "category": "development",
                "summary": "{}",
                "confidence": 0.9,
                "is_work_related": true
            }}]
        }}"#, long_summary);
        let result = validate_vision_result(&json).unwrap();
        assert!(result.items[0].summary.len() <= 80);
    }

    #[test]
    fn test_clamps_confidence() {
        let json = r#"{
            "items": [{
                "category": "development",
                "summary": "test",
                "confidence": 1.5,
                "is_work_related": true
            }]
        }"#;
        let result = validate_vision_result(json).unwrap();
        assert_eq!(result.items[0].confidence, 1.0);
    }

    #[test]
    fn test_unknown_category_becomes_other() {
        let json = r#"{
            "items": [{
                "category": "unknown_category",
                "summary": "test",
                "confidence": 0.5,
                "is_work_related": true
            }]
        }"#;
        let result = validate_vision_result(json).unwrap();
        assert_eq!(result.items[0].category, "other");
    }
}
```

- [ ] **Step 2: Run validation tests**

Run:
```bash
cd src-tauri && cargo test --lib providers::validation
```

Expected: 6 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/providers/
git commit -m "feat: add vision result JSON validation with truncation"
```

---

### Task 12: OpenAI Provider

**Covers:** §6 (vision + text providers)

**Files:**
- Create: `src-tauri/src/providers/openai.rs`
- Modify: `src-tauri/Cargo.toml` (add base64)

- [ ] **Step 1: Write OpenAI provider**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
base64 = "0.22"
```

Create `src-tauri/src/providers/openai.rs`:
```rust
use crate::domain::{VisionProvider, TextProvider, VisionResult};
use crate::providers::validation::validate_vision_result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use log::{debug, warn};

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: serde_json::Value,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: String,
}

fn build_vision_messages(image_data: &[u8]) -> Vec<Message> {
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_data);
    let data_url = format!("data:image/jpeg;base64,{}", b64);

    vec![
        Message {
            role: "system".to_string(),
            content: serde_json::json!({
                "type": "text",
                "text": VISION_SYSTEM_PROMPT
            }),
        },
        Message {
            role: "user".to_string(),
            content: serde_json::json!([{
                "type": "image_url",
                "image_url": { "url": data_url, "detail": "low" }
            }]),
        },
    ]
}

const VISION_SYSTEM_PROMPT: &str = r#"You are a work activity analyzer. Look at this screenshot and describe what work activity is visible.

Rules:
- ONLY describe what is visually visible in the screenshot
- Do NOT infer activities from content you cannot see
- Do NOT read or repeat passwords, API keys, verification codes, or personal sensitive data
- If you cannot determine the activity, use category "other" with low confidence
- Respond in the language of the visible content

Respond with JSON matching this schema:
{
  "items": [
    {
      "category": "development|meeting|communication|documentation|research|design|other",
      "summary": "brief description of visible work (max 80 chars)",
      "detail": "optional additional context (max 240 chars, can be null)",
      "confidence": 0.0 to 1.0,
      "is_work_related": true or false
    }
  ]
}"#;

const TEXT_SYSTEM_PROMPT: &str = r#"You are a professional report writer. Given a list of work activities with timestamps and categories, generate a well-structured Markdown report.

Rules:
- Use the provided activity data as-is, do not invent activities
- Group by category or time period as appropriate
- Include a summary section at the top
- Use proper Markdown formatting with headers, lists, and tables
- Be concise but informative"#;

#[async_trait]
impl VisionProvider for OpenAIProvider {
    async fn analyze_image(&self, image_data: &[u8], model: &str) -> Result<VisionResult, String> {
        let messages = build_vision_messages(image_data);
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens: 1000,
            temperature: 0.1,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let chat_response: ChatResponse = response.json().await
            .map_err(|e| format!("parse response: {}", e))?;

        let content = chat_response.choices.first()
            .ok_or("no choices in response")?
            .message.content.clone();

        debug!("Raw vision response length: {}", content.len());

        validate_vision_result(&content)
    }
}

#[async_trait]
impl TextProvider for OpenAIProvider {
    async fn generate_report(&self, activities_text: &str, template: &str, model: &str) -> Result<String, String> {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: serde_json::json!({
                    "type": "text",
                    "text": format!("{}\n\nUse this template style: {}", TEXT_SYSTEM_PROMPT, template)
                }),
            },
            Message {
                role: "user".to_string(),
                content: serde_json::json!({
                    "type": "text",
                    "text": activities_text
                }),
            },
        ];

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens: 4000,
            temperature: 0.3,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let chat_response: ChatResponse = response.json().await
            .map_err(|e| format!("parse response: {}", e))?;

        Ok(chat_response.choices.first()
            .ok_or("no choices in response")?
            .message.content.clone())
    }
}
```

- [ ] **Step 2: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/providers/openai.rs src-tauri/src/providers/mod.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add OpenAI vision and text provider"
```

---

### Task 13: Anthropic Provider

**Covers:** §6 (vision + text providers)

**Files:**
- Create: `src-tauri/src/providers/anthropic.rs`

- [ ] **Step 1: Write Anthropic provider**

Create `src-tauri/src/providers/anthropic.rs`:
```rust
use crate::domain::{VisionProvider, TextProvider, VisionResult};
use crate::providers::validation::validate_vision_result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use log::debug;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: Option<String>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: serde_json::Value,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

const VISION_SYSTEM_PROMPT: &str = r#"You are a work activity analyzer. Look at this screenshot and describe what work activity is visible.

Rules:
- ONLY describe what is visually visible in the screenshot
- Do NOT infer activities from content you cannot see
- Do NOT read or repeat passwords, API keys, verification codes, or personal sensitive data
- If you cannot determine the activity, use category "other" with low confidence
- Respond in the language of the visible content

Respond with JSON matching this schema:
{
  "items": [
    {
      "category": "development|meeting|communication|documentation|research|design|other",
      "summary": "brief description of visible work (max 80 chars)",
      "detail": "optional additional context (max 240 chars, can be null)",
      "confidence": 0.0 to 1.0,
      "is_work_related": true or false
    }
  ]
}"#;

#[async_trait]
impl VisionProvider for AnthropicProvider {
    async fn analyze_image(&self, image_data: &[u8], model: &str) -> Result<VisionResult, String> {
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_data);

        let messages = vec![Message {
            role: "user".to_string(),
            content: serde_json::json!([
                {
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/jpeg",
                        "data": b64
                    }
                },
                {
                    "type": "text",
                    "text": "Analyze this screenshot and respond with the JSON schema specified in your system prompt."
                }
            ]),
        }];

        let request = MessagesRequest {
            model: model.to_string(),
            max_tokens: 1000,
            messages,
            system: Some(VISION_SYSTEM_PROMPT.to_string()),
        };

        let url = format!("{}/v1/messages", self.base_url);
        let response = self.client.post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let msg_response: MessagesResponse = response.json().await
            .map_err(|e| format!("parse response: {}", e))?;

        let text = msg_response.content.iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text.as_ref())
            .ok_or("no text content in response")?;

        debug!("Raw vision response length: {}", text.len());
        validate_vision_result(text)
    }
}

#[async_trait]
impl TextProvider for AnthropicProvider {
    async fn generate_report(&self, activities_text: &str, template: &str, model: &str) -> Result<String, String> {
        let system = format!(
            "You are a professional report writer. Given work activities, generate a Markdown report.\n\nTemplate style: {}\n\nRules:\n- Use provided data only\n- Group by category/time\n- Include summary\n- Proper Markdown formatting",
            template
        );

        let messages = vec![Message {
            role: "user".to_string(),
            content: serde_json::json!({
                "type": "text",
                "text": activities_text
            }),
        }];

        let request = MessagesRequest {
            model: model.to_string(),
            max_tokens: 4000,
            messages,
            system: Some(system),
        };

        let url = format!("{}/v1/messages", self.base_url);
        let response = self.client.post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let msg_response: MessagesResponse = response.json().await
            .map_err(|e| format!("parse response: {}", e))?;

        msg_response.content.iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text.clone())
            .ok_or("no text content in response".to_string())
    }
}
```

- [ ] **Step 2: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/providers/anthropic.rs
git commit -m "feat: add Anthropic vision and text provider"
```

---

## Phase 5: Pipeline Integration

### Task 14: Retry Logic and Queue Worker

**Covers:** §5 (retry, queue worker)

**Files:**
- Create: `src-tauri/src/pipeline/retry.rs`
- Create: `src-tauri/src/pipeline/queue.rs`

- [ ] **Step 1: Write retry logic**

Create `src-tauri/src/pipeline/retry.rs`:
```rust
use tokio::time::{sleep, Duration};

pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
        }
    }
}

pub async fn with_retry<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                log::warn!("Attempt {}/{} failed: {}", attempt + 1, config.max_attempts, e);
                last_error = Some(e);

                if attempt + 1 < config.max_attempts {
                    let delay = config.base_delay_ms * 2u64.pow(attempt);
                    let delay = delay.min(config.max_delay_ms);
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

pub fn is_retryable_error(error: &str) -> bool {
    error.contains("429")
        || error.contains("500")
        || error.contains("502")
        || error.contains("503")
        || error.contains("timeout")
        || error.contains("connection")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_succeeds_on_first_try() {
        let config = RetryConfig::default();
        let result = with_retry(&config, || async { Ok::<_, String>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failure() {
        let config = RetryConfig {
            base_delay_ms: 1,
            ..Default::default()
        };
        let attempt = std::sync::atomic::AtomicU32::new(0);
        let result = with_retry(&config, || {
            let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move {
                if n == 0 {
                    Err("temporary failure".to_string())
                } else {
                    Ok(42)
                }
            }
        }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_exhausts_attempts() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay_ms: 1,
            ..Default::default()
        };
        let result: Result<(), String> = with_retry(&config, || async {
            Err("permanent failure".to_string())
        }).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable_error("429 Too Many Requests"));
        assert!(is_retryable_error("500 Internal Server Error"));
        assert!(!is_retryable_error("400 Bad Request"));
        assert!(!is_retryable_error("invalid JSON"));
    }
}
```

- [ ] **Step 2: Write queue worker**

Create `src-tauri/src/pipeline/queue.rs`:
```rust
use crate::domain::{CaptureProvider, VisionProvider};
use crate::storage::Database;
use crate::pipeline::retry::{RetryConfig, with_retry, is_retryable_error};
use crate::pipeline::dedup::DedupChecker;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use log::{info, warn, error};

pub struct QueueWorker {
    db: Arc<Database>,
    dedup: DedupChecker,
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
        frame_data: &[u8],
        display_id: &str,
        vision_provider: &dyn VisionProvider,
        model: &str,
        provider_name: &str,
    ) -> Result<(), String> {
        // Check dedup
        if self.dedup.check_and_update(frame_data)? {
            info!("Skipping duplicate frame for display {}", display_id);
            return Ok(());
        }

        // Compute hash for tracking
        use md5::Digest;
        let hash = format!("{:x}", md5::Md5::digest(frame_data));

        // Create job
        let job_id = uuid::Uuid::new_v4().to_string();
        let job = crate::domain::AnalysisJob {
            id: job_id.clone(),
            captured_at: chrono::Utc::now(),
            status: crate::domain::JobStatus::Pending,
            attempts: 0,
            last_error: None,
            image_hash: Some(hash),
            provider: Some(provider_name.to_string()),
            model: Some(model.to_string()),
            created_at: chrono::Utc::now(),
            finished_at: None,
        };
        self.db.insert_job(&job)?;

        // Process with retry
        let retry_config = RetryConfig::default();
        let result = with_retry(&retry_config, || async {
            match vision_provider.analyze_image(frame_data, model).await {
                Ok(vision_result) => Ok(vision_result),
                Err(e) => {
                    if is_retryable_error(&e) {
                        Err(e)
                    } else {
                        Err(e)
                    }
                }
            }
        }).await;

        match result {
            Ok(vision_result) => {
                // Create activities from vision result
                for item in vision_result.items {
                    let activity = crate::domain::Activity {
                        id: uuid::Uuid::new_v4().to_string(),
                        job_id: job_id.clone(),
                        started_at: chrono::Utc::now(),
                        ended_at: chrono::Utc::now(),
                        category: crate::domain::Category::from_str(&item.category),
                        summary: item.summary,
                        detail: item.detail,
                        confidence: item.confidence,
                        is_work_related: item.is_work_related,
                        source: "auto".to_string(),
                        edited_at: None,
                        deleted_at: None,
                    };
                    self.db.insert_activity(&activity)?;
                }
                self.db.complete_job(&job_id, provider_name, model)?;
                info!("Job {} completed successfully", job_id);
                Ok(())
            }
            Err(e) => {
                error!("Job {} failed after retries: {}", job_id, e);
                self.db.fail_job(&job_id, &e)?;
                Err(e)
            }
        }
    }

    pub fn reset_dedup(&mut self) {
        self.dedup.reset();
    }
}
```

- [ ] **Step 3: Add md5 dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
md-5 = "0.10"
```

- [ ] **Step 4: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/pipeline/retry.rs src-tauri/src/pipeline/queue.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add retry logic and queue worker for AI analysis"
```

---

### Task 15: Scheduler with Real Tick Loop

**Covers:** §5 (scheduler workflow)

**Files:**
- Modify: `src-tauri/src/pipeline/scheduler.rs`

- [ ] **Step 1: Add run loop to scheduler**

Add to `src-tauri/src/pipeline/scheduler.rs`:
```rust
use crate::domain::CaptureProvider;
use crate::pipeline::queue::QueueWorker;
use std::sync::Arc;
use tokio::sync::watch;
use log::{info, warn};

impl CaptureScheduler {
    pub async fn run(
        &self,
        mut capture: Box<dyn CaptureProvider>,
        vision_provider: Arc<dyn crate::domain::VisionProvider>,
        model: String,
        provider_name: String,
        mut queue_worker: QueueWorker,
        mut idle_rx: watch::Receiver<bool>,
    ) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.interval_sec as u64)
        );
        interval.tick().await; // skip first immediate tick

        let mut state_rx = self.state_rx();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let state = state_rx.borrow_and_update().clone();
                    let is_idle = *idle_rx.borrow_and_update();

                    if state != RecordingState::Recording {
                        continue;
                    }

                    if is_idle {
                        info!("System idle, skipping capture");
                        continue;
                    }

                    match capture.capture_frame().await {
                        Ok(frame) => {
                            if let Err(e) = queue_worker.process_frame(
                                &frame.data,
                                &frame.display_id,
                                vision_provider.as_ref(),
                                &model,
                                &provider_name,
                            ).await {
                                warn!("Failed to process frame: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Capture failed: {}", e);
                        }
                    }
                }
                _ = state_rx.changed() => {
                    let new_state = state_rx.borrow_and_update().clone();
                    info!("Scheduler: state changed to {:?}", new_state);
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pipeline/scheduler.rs
git commit -m "feat: add scheduler run loop with idle detection integration"
```

---

## Phase 6: Platform Services

### Task 16: Idle Detection via D-Bus

**Covers:** §2 (idle detection)

**Files:**
- Create: `src-tauri/src/platform/mod.rs`
- Create: `src-tauri/src/platform/idle.rs`
- Create: `src-tauri/src/platform/paths.rs`

- [ ] **Step 1: Write idle detection**

Create `src-tauri/src/platform/mod.rs`:
```rust
pub mod idle;
pub mod paths;
pub mod tray;
pub mod notifications;
```

Create `src-tauri/src/platform/idle.rs`:
```rust
use tokio::sync::watch;
use log::{debug, warn};

pub struct IdleDetector {
    idle_tx: watch::Sender<bool>,
    idle_rx: watch::Receiver<bool>,
    threshold_sec: u32,
}

impl IdleDetector {
    pub fn new(threshold_sec: u32) -> Self {
        let (idle_tx, idle_rx) = watch::channel(false);
        Self {
            idle_tx,
            idle_rx,
            threshold_sec,
        }
    }

    pub fn is_idle(&self) -> bool {
        *self.idle_rx.borrow()
    }

    pub fn idle_rx(&self) -> watch::Receiver<bool> {
        self.idle_rx.clone()
    }

    pub async fn run(&self) {
        // Try D-Bus idle detection (works on GNOME, KDE, etc.)
        match self.run_dbus().await {
            Ok(()) => {},
            Err(e) => {
                warn!("D-Bus idle detection unavailable: {}. Falling back to polling.", e);
                self.run_polling().await;
            }
        }
    }

    async fn run_dbus(&self) -> Result<(), String> {
        // Use org.freedesktop.ScreenSaver for idle detection
        // This works on most Linux desktops
        let conn = zbus::Connection::session().await
            .map_err(|e| format!("D-Bus connection failed: {}", e))?;

        loop {
            let idle_time = get_idle_time_dbus(&conn).await.unwrap_or(0);
            let is_idle = idle_time >= self.threshold_sec as u64;

            if is_idle != *self.idle_rx.borrow() {
                debug!("Idle state changed: {}", is_idle);
                self.idle_tx.send(is_idle).ok();
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    async fn run_polling(&self) {
        // Fallback: assume not idle
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}

async fn get_idle_time_dbus(conn: &zbus::Connection) -> Result<u64, String> {
    let proxy = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.ScreenSaver")
        .map_err(|e| format!("proxy dest: {}", e))?
        .path("/org/freedesktop/ScreenSaver")
        .map_err(|e| format!("proxy path: {}", e))?
        .interface("org.freedesktop.ScreenSaver")
        .map_err(|e| format!("proxy iface: {}", e))?
        .build()
        .await
        .map_err(|e| format!("proxy build: {}", e))?;

    let idle_ms: u32 = proxy.call("GetSessionIdleTime", &())
        .await
        .map_err(|e| format!("call GetSessionIdleTime: {}", e))?;

    Ok(idle_ms as u64 / 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_not_idle() {
        let detector = IdleDetector::new(300);
        assert!(!detector.is_idle());
    }
}
```

Create `src-tauri/src/platform/paths.rs`:
```rust
use std::path::PathBuf;

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("daytrace")
}

pub fn app_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("daytrace")
}

pub fn db_path() -> PathBuf {
    app_data_dir().join("daytrace.db")
}

pub fn temp_image_dir() -> PathBuf {
    app_cache_dir().join("screenshots")
}

pub fn ensure_dirs() -> Result<(), String> {
    let dirs = vec![app_data_dir(), app_cache_dir(), temp_image_dir()];
    for dir in dirs {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("create dir {}: {}", dir.display(), e))?;
    }

    // Set temp dir permissions to 0700
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(temp_image_dir(), perms)
            .map_err(|e| format!("set permissions: {}", e))?;
    }

    Ok(())
}

pub fn cleanup_temp_files() -> Result<u32, String> {
    let temp_dir = temp_image_dir();
    if !temp_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in std::fs::read_dir(&temp_dir)
        .map_err(|e| format!("read temp dir: {}", e))?
    {
        let entry = entry.map_err(|e| format!("read entry: {}", e))?;
        if entry.path().is_file() {
            std::fs::remove_file(entry.path())
                .map_err(|e| format!("remove {}: {}", entry.path().display(), e))?;
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_are_valid() {
        let data = app_data_dir();
        let cache = app_cache_dir();
        assert!(data.to_string_lossy().contains("daytrace"));
        assert!(cache.to_string_lossy().contains("daytrace"));
    }

    #[test]
    fn test_ensure_dirs_creates_directories() {
        ensure_dirs().unwrap();
        assert!(app_data_dir().exists());
        assert!(app_cache_dir().exists());
        assert!(temp_image_dir().exists());
    }
}
```

- [ ] **Step 2: Add dirs dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
dirs = "6"
```

- [ ] **Step 3: Update lib.rs**

Update `src-tauri/src/lib.rs`:
```rust
pub mod storage;
pub mod domain;
pub mod capture;
pub mod pipeline;
pub mod providers;
pub mod platform;
```

- [ ] **Step 4: Run tests**

Run:
```bash
cd src-tauri && cargo test --lib platform
```

Expected: All platform tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/platform/ src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add idle detection, app paths, and temp file cleanup"
```

---

## Phase 7: Tauri Commands and Frontend

### Task 17: Tauri Command Layer

**Covers:** §9 (thin Tauri command adapters)

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write Tauri commands**

Create `src-tauri/src/commands/mod.rs`:
```rust
use crate::domain::{Activity, Category, Report, PeriodType};
use crate::storage::Database;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Serialize)]
pub struct TodayStats {
    pub total_activities: usize,
    pub work_activities: usize,
    pub categories: Vec<CategoryCount>,
    pub failed_jobs: i64,
}

#[derive(Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: usize,
}

#[tauri::command]
pub async fn get_today(db: State<'_, Arc<Database>>) -> Result<(Vec<Activity>, TodayStats), String> {
    let today = chrono::Utc::now().date_naive();
    let activities = db.get_activities_for_date(today)?;

    let mut categories = std::collections::HashMap::new();
    for a in &activities {
        *categories.entry(a.category.as_str().to_string()).or_insert(0) += 1;
    }

    let stats = TodayStats {
        total_activities: activities.len(),
        work_activities: activities.iter().filter(|a| a.is_work_related).count(),
        categories: categories.into_iter()
            .map(|(category, count)| CategoryCount { category, count })
            .collect(),
        failed_jobs: 0,
    };

    Ok((activities, stats))
}

#[derive(Deserialize)]
pub struct UpdateActivityRequest {
    pub id: String,
    pub summary: Option<String>,
    pub category: Option<String>,
    pub detail: Option<String>,
    pub is_work_related: Option<bool>,
}

#[tauri::command]
pub async fn update_activity(
    db: State<'_, Arc<Database>>,
    request: UpdateActivityRequest,
) -> Result<(), String> {
    let today = chrono::Utc::now().date_naive();
    let activities = db.get_activities_for_date(today)?;

    let mut activity = activities.into_iter()
        .find(|a| a.id == request.id)
        .ok_or("activity not found")?;

    if let Some(summary) = request.summary {
        activity.summary = summary;
    }
    if let Some(category) = request.category {
        activity.category = Category::from_str(&category);
    }
    if let Some(detail) = request.detail {
        activity.detail = Some(detail);
    }
    if let Some(is_work) = request.is_work_related {
        activity.is_work_related = is_work;
    }

    db.update_activity(&activity)
}

#[tauri::command]
pub async fn delete_activity(db: State<'_, Arc<Database>>, id: String) -> Result<(), String> {
    db.soft_delete_activity(&id)
}

#[tauri::command]
pub async fn generate_report(
    db: State<'_, Arc<Database>>,
    period_type: String,
    period_start: String,
    template_id: String,
) -> Result<Report, String> {
    let start = NaiveDate::parse_from_str(&period_start, "%Y-%m-%d")
        .map_err(|e| format!("invalid date: {}", e))?;

    let (end, title) = match period_type.as_str() {
        "daily" => (start, format!("Daily Report - {}", start)),
        "weekly" => {
            let end = start + chrono::Duration::days(6);
            (end, format!("Weekly Report - {} to {}", start, end))
        }
        "monthly" => {
            let end = start.with_day(1)
                .and_then(|d| d.checked_add_months(chrono::Months::new(1)))
                .and_then(|d| d.pred_opt())
                .unwrap_or(start);
            (end, format!("Monthly Report - {}", start.format("%B %Y")))
        }
        _ => return Err("invalid period type".to_string()),
    };

    let activities = db.get_activities_in_range(start, end)?;

    if activities.is_empty() {
        return Err("no activities found for this period".to_string());
    }

    // Aggregate locally first
    let aggregated = crate::reporting::aggregation::aggregate_activities(&activities);

    let report = Report {
        id: uuid::Uuid::new_v4().to_string(),
        period_type: PeriodType::from_str(&period_type),
        period_start: start,
        period_end: end,
        template_id,
        title,
        content_markdown: aggregated,
        model: None,
        prompt_version: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    db.insert_report(&report)?;
    Ok(report)
}

#[tauri::command]
pub async fn list_reports(
    db: State<'_, Arc<Database>>,
    period_type: Option<String>,
) -> Result<Vec<Report>, String> {
    let pt = period_type.map(|s| PeriodType::from_str(&s));
    db.get_reports(pt)
}

#[tauri::command]
pub async fn start_recording(
    scheduler: State<'_, Arc<crate::pipeline::scheduler::CaptureScheduler>>,
) -> Result<(), String> {
    scheduler.start()
}

#[tauri::command]
pub async fn pause_recording(
    scheduler: State<'_, Arc<crate::pipeline::scheduler::CaptureScheduler>>,
) -> Result<(), String> {
    scheduler.pause()
}

#[tauri::command]
pub async fn stop_recording(
    scheduler: State<'_, Arc<crate::pipeline::scheduler::CaptureScheduler>>,
) -> Result<(), String> {
    scheduler.stop()
}

#[tauri::command]
pub async fn get_recording_state(
    scheduler: State<'_, Arc<crate::pipeline::scheduler::CaptureScheduler>>,
) -> Result<String, String> {
    Ok(format!("{:?}", scheduler.state()))
}

#[tauri::command]
pub async fn save_provider_key(service: String, key: String) -> Result<(), String> {
    let entry = keyring::Entry::new("daytrace", &service)
        .map_err(|e| format!("keyring entry: {}", e))?;
    entry.set_password(&key)
        .map_err(|e| format!("save key: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn test_provider_key(service: String) -> Result<bool, String> {
    let entry = keyring::Entry::new("daytrace", &service)
        .map_err(|e| format!("keyring entry: {}", e))?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("keyring error: {}", e)),
    }
}

#[tauri::command]
pub async fn get_settings(db: State<'_, Arc<Database>>) -> Result<crate::domain::AppSettings, String> {
    db.get_settings()
}

#[tauri::command]
pub async fn save_settings(
    db: State<'_, Arc<Database>>,
    settings: crate::domain::AppSettings,
) -> Result<(), String> {
    db.save_settings(&settings)
}

#[tauri::command]
pub async fn clear_all_data(db: State<'_, Arc<Database>>) -> Result<(), String> {
    let conn = db.conn();
    conn.execute_batch(
        "DELETE FROM api_usage; DELETE FROM activities; DELETE FROM analysis_jobs; DELETE FROM reports; DELETE FROM capture_sessions;"
    ).map_err(|e| format!("clear data: {}", e))?;

    crate::platform::paths::cleanup_temp_files()?;
    Ok(())
}

#[tauri::command]
pub async fn get_daily_usage(db: State<'_, Arc<Database>>) -> Result<i64, String> {
    let today = chrono::Utc::now().date_naive();
    db.get_daily_usage_cents(today)
}
```

- [ ] **Step 2: Add reporting module placeholder**

Create `src-tauri/src/reporting/mod.rs`:
```rust
pub mod aggregation;
pub mod templates;
pub mod export;
```

Create `src-tauri/src/reporting/aggregation.rs`:
```rust
use crate::domain::Activity;
use std::collections::HashMap;

pub fn aggregate_activities(activities: &[Activity]) -> String {
    let mut by_category: HashMap<String, Vec<&Activity>> = HashMap::new();

    for activity in activities {
        by_category
            .entry(activity.category.as_str().to_string())
            .or_default()
            .push(activity);
    }

    let mut output = String::new();
    output.push_str("# Work Activities\n\n");
    output.push_str(&format!("**Total activities:** {}\n\n", activities.len()));

    for (category, items) in &by_category {
        output.push_str(&format!("## {}\n\n", capitalize(category)));
        for item in items {
            output.push_str(&format!("- {}", item.summary));
            if let Some(ref detail) = item.detail {
                output.push_str(&format!(" ({})", detail));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    output
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Activity, Category};
    use chrono::Utc;

    fn make_activity(summary: &str, category: Category) -> Activity {
        Activity {
            id: "1".to_string(),
            job_id: "j1".to_string(),
            started_at: Utc::now(),
            ended_at: Utc::now(),
            category,
            summary: summary.to_string(),
            detail: None,
            confidence: 0.9,
            is_work_related: true,
            source: "auto".to_string(),
            edited_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_aggregate_empty() {
        let result = aggregate_activities(&[]);
        assert!(result.contains("Total activities:** 0"));
    }

    #[test]
    fn test_aggregate_groups_by_category() {
        let activities = vec![
            make_activity("Coding", Category::Development),
            make_activity("Meeting", Category::Meeting),
            make_activity("More coding", Category::Development),
        ];
        let result = aggregate_activities(&activities);
        assert!(result.contains("Development"));
        assert!(result.contains("Meeting"));
    }
}
```

Create `src-tauri/src/reporting/templates.rs`:
```rust
pub const TEMPLATES: &[(&str, &str)] = &[
    ("standard", "Standard daily report with categories and timeline"),
    ("concise", "Brief bullet-point summary"),
    ("technical", "Technical focus with code/implementation details"),
    ("okr", "OKR-aligned progress report"),
];

pub fn get_template_prompt(template_id: &str) -> &str {
    match template_id {
        "concise" => "Write a brief bullet-point report. Use short phrases, no paragraphs. Group by category.",
        "technical" => "Write a technical report focusing on implementation details, code changes, and technical decisions. Use code blocks where appropriate.",
        "okr" => "Write an OKR-aligned report. Group activities by objective/key result. Include progress indicators.",
        _ => "Write a standard professional daily report with clear categories, timeline, and summary.",
    }
}
```

Create `src-tauri/src/reporting/export.rs`:
```rust
use std::path::PathBuf;

pub fn export_markdown(content: &str, filename: &str, export_dir: &PathBuf) -> Result<PathBuf, String> {
    std::fs::create_dir_all(export_dir)
        .map_err(|e| format!("create export dir: {}", e))?;

    let path = export_dir.join(format!("{}.md", filename));
    std::fs::write(&path, content)
        .map_err(|e| format!("write report: {}", e))?;

    Ok(path)
}
```

- [ ] **Step 3: Update main.rs with all commands**

Update `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // Initialize database
            let db_path = daytrace::platform::paths::db_path();
            daytrace::platform::paths::ensure_dirs().map_err(|e| e.to_string())?;
            daytrace::platform::paths::cleanup_temp_files().map_err(|e| e.to_string())?;

            let db = Arc::new(
                daytrace::storage::Database::new(&db_path)
                    .map_err(|e| e.to_string())?
            );

            // Initialize scheduler
            let settings = db.get_settings().unwrap_or_default();
            let scheduler = Arc::new(
                daytrace::pipeline::scheduler::CaptureScheduler::new(
                    settings.capture_interval_sec,
                    settings.idle_threshold_sec,
                )
            );

            app.manage(db);
            app.manage(scheduler);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            daytrace::commands::get_today,
            daytrace::commands::update_activity,
            daytrace::commands::delete_activity,
            daytrace::commands::generate_report,
            daytrace::commands::list_reports,
            daytrace::commands::start_recording,
            daytrace::commands::pause_recording,
            daytrace::commands::stop_recording,
            daytrace::commands::get_recording_state,
            daytrace::commands::save_provider_key,
            daytrace::commands::test_provider_key,
            daytrace::commands::get_settings,
            daytrace::commands::save_settings,
            daytrace::commands::clear_all_data,
            daytrace::commands::get_daily_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Update `src-tauri/src/lib.rs`:
```rust
pub mod storage;
pub mod domain;
pub mod capture;
pub mod pipeline;
pub mod providers;
pub mod platform;
pub mod commands;
pub mod reporting;
```

- [ ] **Step 4: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/reporting/ src-tauri/src/main.rs src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands and report aggregation"
```

---

### Task 18: Frontend API Layer and Types

**Covers:** §9 (typed invoke wrappers)

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/constants.ts`
- Create: `src/api/tauri.ts`

- [ ] **Step 1: Write TypeScript types**

Create `src/lib/types.ts`:
```typescript
export type Category =
  | 'development'
  | 'meeting'
  | 'communication'
  | 'documentation'
  | 'research'
  | 'design'
  | 'other';

export type RecordingState = 'Stopped' | 'Recording' | 'Paused';

export type PeriodType = 'daily' | 'weekly' | 'monthly';

export interface Activity {
  id: string;
  job_id: string;
  started_at: string;
  ended_at: string;
  category: Category;
  summary: string;
  detail: string | null;
  confidence: number;
  is_work_related: boolean;
  source: string;
  edited_at: string | null;
  deleted_at: string | null;
}

export interface Report {
  id: string;
  period_type: PeriodType;
  period_start: string;
  period_end: string;
  template_id: string;
  title: string;
  content_markdown: string;
  model: string | null;
  prompt_version: string | null;
  created_at: string;
  updated_at: string;
}

export interface TodayStats {
  total_activities: number;
  work_activities: number;
  categories: CategoryCount[];
  failed_jobs: number;
}

export interface CategoryCount {
  category: Category;
  count: number;
}

export interface AppSettings {
  capture_interval_sec: number;
  idle_threshold_sec: number;
  selected_displays: string[];
  provider: ProviderConfig;
  daily_budget_cents: number;
  privacy_mode: boolean;
  hash_similarity_threshold: number;
}

export interface ProviderConfig {
  vision_provider: string;
  vision_model: string;
  text_provider: string;
  text_model: string;
  base_url_overrides: Record<string, string>;
}

export interface UpdateActivityRequest {
  id: string;
  summary?: string;
  category?: Category;
  detail?: string;
  is_work_related?: boolean;
}
```

Create `src/lib/constants.ts`:
```typescript
import { Category } from './types';

export const CATEGORIES: { value: Category; label: string }[] = [
  { value: 'development', label: 'Development' },
  { value: 'meeting', label: 'Meeting' },
  { value: 'communication', label: 'Communication' },
  { value: 'documentation', label: 'Documentation' },
  { value: 'research', label: 'Research' },
  { value: 'design', label: 'Design' },
  { value: 'other', label: 'Other' },
];

export const CATEGORY_COLORS: Record<Category, string> = {
  development: '#3B82F6',
  meeting: '#F59E0B',
  communication: '#10B981',
  documentation: '#8B5CF6',
  research: '#EC4899',
  design: '#F97316',
  other: '#6B7280',
};

export const TEMPLATES = [
  { id: 'standard', label: 'Standard' },
  { id: 'concise', label: 'Concise' },
  { id: 'technical', label: 'Technical' },
  { id: 'okr', label: 'OKR' },
];

export const PROVIDERS = [
  { id: 'openai', label: 'OpenAI' },
  { id: 'anthropic', label: 'Anthropic' },
];

export const DEFAULT_SETTINGS: AppSettings = {
  capture_interval_sec: 300,
  idle_threshold_sec: 300,
  selected_displays: [],
  provider: {
    vision_provider: 'openai',
    vision_model: 'gpt-4o',
    text_provider: 'openai',
    text_model: 'gpt-4o',
    base_url_overrides: {},
  },
  daily_budget_cents: 500,
  privacy_mode: false,
  hash_similarity_threshold: 0.9,
};

import type { AppSettings } from './types';
```

Create `src/api/tauri.ts`:
```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  Activity,
  Report,
  TodayStats,
  AppSettings,
  UpdateActivityRequest,
  RecordingState,
} from '../lib/types';

export async function getToday(): Promise<[Activity[], TodayStats]> {
  return invoke('get_today');
}

export async function updateActivity(request: UpdateActivityRequest): Promise<void> {
  return invoke('update_activity', { request });
}

export async function deleteActivity(id: string): Promise<void> {
  return invoke('delete_activity', { id });
}

export async function generateReport(
  periodType: string,
  periodStart: string,
  templateId: string
): Promise<Report> {
  return invoke('generate_report', {
    periodType,
    periodStart,
    templateId,
  });
}

export async function listReports(periodType?: string): Promise<Report[]> {
  return invoke('list_reports', { periodType: periodType ?? null });
}

export async function startRecording(): Promise<void> {
  return invoke('start_recording');
}

export async function pauseRecording(): Promise<void> {
  return invoke('pause_recording');
}

export async function stopRecording(): Promise<void> {
  return invoke('stop_recording');
}

export async function getRecordingState(): Promise<RecordingState> {
  return invoke('get_recording_state');
}

export async function saveProviderKey(service: string, key: string): Promise<void> {
  return invoke('save_provider_key', { service, key });
}

export async function testProviderKey(service: string): Promise<boolean> {
  return invoke('test_provider_key', { service });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings });
}

export async function clearAllData(): Promise<void> {
  return invoke('clear_all_data');
}

export async function getDailyUsage(): Promise<number> {
  return invoke('get_daily_usage');
}

export function onRecordingStatus(callback: (status: RecordingState) => void) {
  return listen<RecordingState>('recording-status', (event) => {
    callback(event.payload);
  });
}

export function onActivityCreated(callback: (activity: Activity) => void) {
  return listen<Activity>('activity-created', (event) => {
    callback(event.payload);
  });
}

export function onJobUpdated(callback: (job: { id: string; status: string }) => void) {
  return listen<{ id: string; status: string }>('job-updated', (event) => {
    callback(event.payload);
  });
}
```

- [ ] **Step 2: Verify frontend builds**

Run:
```bash
npm run build
```

Expected: No TypeScript errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/ src/api/
git commit -m "feat: add frontend types, constants, and Tauri API wrappers"
```

---

### Task 19: Today Page with Timeline

**Covers:** §8 (today page, timeline, activity cards)

**Files:**
- Create: `src/stores/recording.ts`
- Create: `src/components/CaptureToggle.tsx`
- Create: `src/components/Timeline.tsx`
- Create: `src/components/ActivityCard.tsx`
- Create: `src/components/StatusBadge.tsx`
- Create: `src/components/BudgetIndicator.tsx`
- Modify: `src/pages/Today.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Write recording store**

Create `src/stores/recording.ts`:
```typescript
import { create } from 'zustand';
import type { RecordingState } from '../lib/types';

interface RecordingStore {
  state: RecordingState;
  setState: (state: RecordingState) => void;
}

export const useRecordingStore = create<RecordingStore>((set) => ({
  state: 'Stopped',
  setState: (state) => set({ state }),
}));
```

- [ ] **Step 2: Write CaptureToggle component**

Create `src/components/CaptureToggle.tsx`:
```tsx
import { useRecordingStore } from '../stores/recording';
import { startRecording, pauseRecording, stopRecording } from '../api/tauri';

export function CaptureToggle() {
  const { state } = useRecordingStore();

  const handleToggle = async () => {
    try {
      if (state === 'Stopped') {
        await startRecording();
      } else if (state === 'Recording') {
        await pauseRecording();
      } else if (state === 'Paused') {
        await startRecording();
      }
    } catch (e) {
      console.error('Failed to toggle recording:', e);
    }
  };

  const handleStop = async () => {
    try {
      await stopRecording();
    } catch (e) {
      console.error('Failed to stop recording:', e);
    }
  };

  return (
    <div className="flex items-center gap-2">
      <button
        onClick={handleToggle}
        className={`px-4 py-2 rounded font-medium ${
          state === 'Recording'
            ? 'bg-yellow-500 hover:bg-yellow-600 text-white'
            : 'bg-green-500 hover:bg-green-600 text-white'
        }`}
      >
        {state === 'Stopped' && 'Start Recording'}
        {state === 'Recording' && 'Pause'}
        {state === 'Paused' && 'Resume'}
      </button>
      {(state === 'Recording' || state === 'Paused') && (
        <button
          onClick={handleStop}
          className="px-4 py-2 rounded bg-red-500 hover:bg-red-600 text-white"
        >
          Stop
        </button>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Write StatusBadge component**

Create `src/components/StatusBadge.tsx`:
```tsx
import { useRecordingStore } from '../stores/recording';

export function StatusBadge() {
  const { state } = useRecordingStore();

  const colors = {
    Stopped: 'bg-gray-400',
    Recording: 'bg-green-500 animate-pulse',
    Paused: 'bg-yellow-500',
  };

  return (
    <span className={`inline-flex items-center gap-1.5 px-2 py-1 rounded text-sm text-white ${colors[state]}`}>
      <span className="w-2 h-2 rounded-full bg-white" />
      {state}
    </span>
  );
}
```

- [ ] **Step 4: Write BudgetIndicator component**

Create `src/components/BudgetIndicator.tsx`:
```tsx
import { useQuery } from '@tanstack/react-query';
import { getDailyUsage, getSettings } from '../api/tauri';

export function BudgetIndicator() {
  const { data: usage } = useQuery({
    queryKey: ['dailyUsage'],
    queryFn: getDailyUsage,
    refetchInterval: 60000,
  });

  const { data: settings } = useQuery({
    queryKey: ['settings'],
    queryFn: getSettings,
  });

  const spent = (usage ?? 0) / 100;
  const budget = (settings?.daily_budget_cents ?? 500) / 100;
  const percent = budget > 0 ? Math.min(100, (spent / budget) * 100) : 0;

  return (
    <div className="text-sm">
      <span className="text-gray-600">Today: </span>
      <span className={percent > 80 ? 'text-red-600 font-medium' : 'text-gray-900'}>
        ${spent.toFixed(2)}
      </span>
      <span className="text-gray-400"> / ${budget.toFixed(2)}</span>
      <div className="w-24 h-1.5 bg-gray-200 rounded mt-1">
        <div
          className={`h-full rounded ${percent > 80 ? 'bg-red-500' : 'bg-blue-500'}`}
          style={{ width: `${percent}%` }}
        />
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Write Timeline and ActivityCard components**

Create `src/components/ActivityCard.tsx`:
```tsx
import type { Activity } from '../lib/types';
import { CATEGORY_COLORS } from '../lib/constants';

interface Props {
  activity: Activity;
  onEdit: (activity: Activity) => void;
  onDelete: (id: string) => void;
}

export function ActivityCard({ activity, onEdit, onDelete }: Props) {
  const time = new Date(activity.started_at).toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  });

  return (
    <div className="border rounded-lg p-3 hover:shadow-sm transition-shadow">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-2">
          <span
            className="w-3 h-3 rounded-full flex-shrink-0"
            style={{ backgroundColor: CATEGORY_COLORS[activity.category] }}
          />
          <span className="text-xs text-gray-500">{time}</span>
          <span className="text-xs px-1.5 py-0.5 rounded bg-gray-100 text-gray-600">
            {activity.category}
          </span>
        </div>
        <div className="flex gap-1">
          <button
            onClick={() => onEdit(activity)}
            className="text-xs text-blue-600 hover:text-blue-800"
          >
            Edit
          </button>
          <button
            onClick={() => onDelete(activity.id)}
            className="text-xs text-red-600 hover:text-red-800"
          >
            Delete
          </button>
        </div>
      </div>
      <p className="mt-1 text-sm">{activity.summary}</p>
      {activity.detail && (
        <p className="mt-1 text-xs text-gray-500">{activity.detail}</p>
      )}
    </div>
  );
}
```

Create `src/components/Timeline.tsx`:
```tsx
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getToday, deleteActivity } from '../api/tauri';
import { ActivityCard } from './ActivityCard';
import type { Activity } from '../lib/types';

interface Props {
  onEdit: (activity: Activity) => void;
}

export function Timeline({ onEdit }: Props) {
  const queryClient = useQueryClient();
  const { data, isLoading } = useQuery({
    queryKey: ['today'],
    queryFn: getToday,
    refetchInterval: 30000,
  });

  const deleteMutation = useMutation({
    mutationFn: deleteActivity,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['today'] }),
  });

  if (isLoading) return <div className="text-gray-500">Loading...</div>;

  const [activities, stats] = data ?? [[], { total_activities: 0, work_activities: 0, categories: [], failed_jobs: 0 }];

  if (activities.length === 0) {
    return <div className="text-gray-500 text-center py-8">No activities recorded today</div>;
  }

  return (
    <div className="space-y-3">
      <div className="flex gap-4 text-sm text-gray-600 mb-4">
        <span>{stats.total_activities} activities</span>
        <span>{stats.work_activities} work-related</span>
        {stats.failed_jobs > 0 && (
          <span className="text-red-600">{stats.failed_jobs} failed</span>
        )}
      </div>
      <div className="space-y-2">
        {activities.map((activity) => (
          <ActivityCard
            key={activity.id}
            activity={activity}
            onEdit={onEdit}
            onDelete={(id) => deleteMutation.mutate(id)}
          />
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 6: Write Today page**

Create `src/pages/Today.tsx` (replace placeholder):
```tsx
import { useState } from 'react';
import { CaptureToggle } from '../components/CaptureToggle';
import { StatusBadge } from '../components/StatusBadge';
import { BudgetIndicator } from '../components/BudgetIndicator';
import { Timeline } from '../components/Timeline';
import type { Activity } from '../lib/types';

export function Today() {
  const [editingActivity, setEditingActivity] = useState<Activity | null>(null);

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <CaptureToggle />
          <StatusBadge />
        </div>
        <BudgetIndicator />
      </div>
      <h2 className="text-lg font-semibold mb-4">Today's Timeline</h2>
      <Timeline onEdit={setEditingActivity} />
    </div>
  );
}
```

- [ ] **Step 7: Update App.tsx**

Replace `src/App.tsx`:
```tsx
import { BrowserRouter, Routes, Route, NavLink } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Today } from './pages/Today';
import { Reports } from './pages/Reports';
import { Settings } from './pages/Settings';

const queryClient = new QueryClient();

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="flex h-screen bg-gray-50">
          <nav className="w-48 bg-white border-r p-4 flex flex-col gap-1">
            <h1 className="text-lg font-bold mb-4">Daytrace</h1>
            <NavLink
              to="/"
              className={({ isActive }) =>
                `px-3 py-2 rounded text-sm ${isActive ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'}`
              }
            >
              Today
            </NavLink>
            <NavLink
              to="/reports"
              className={({ isActive }) =>
                `px-3 py-2 rounded text-sm ${isActive ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'}`
              }
            >
              Reports
            </NavLink>
            <NavLink
              to="/settings"
              className={({ isActive }) =>
                `px-3 py-2 rounded text-sm ${isActive ? 'bg-blue-100 text-blue-700' : 'hover:bg-gray-100'}`
              }
            >
              Settings
            </NavLink>
          </nav>
          <main className="flex-1 p-6 overflow-auto">
            <Routes>
              <Route path="/" element={<Today />} />
              <Route path="/reports" element={<Reports />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
```

Create placeholder pages:
Create `src/pages/Reports.tsx`:
```tsx
export function Reports() {
  return <div>Reports - Coming soon</div>;
}
```

Create `src/pages/Settings.tsx`:
```tsx
export function Settings() {
  return <div>Settings - Coming soon</div>;
}
```

- [ ] **Step 8: Verify frontend builds**

Run:
```bash
npm run build
```

Expected: No errors.

- [ ] **Step 9: Commit**

```bash
git add src/stores/ src/components/ src/pages/ src/App.tsx
git commit -m "feat: add Today page with timeline, capture toggle, and budget indicator"
```

---

## Phase 8: Integration and Polish

### Task 20: X11 Capture Provider

**Covers:** §4 (X11 capture)

**Files:**
- Create: `src-tauri/src/capture/x11.rs`
- Modify: `src-tauri/src/capture/mod.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add xcap dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
xcap = "0.6"
```

- [ ] **Step 2: Write X11 capture provider**

Create `src-tauri/src/capture/x11.rs`:
```rust
use crate::domain::capture::{CaptureCapabilities, CaptureProvider, CapturedFrame};
use async_trait::async_trait;
use image::ImageEncoder;
use log::info;

pub struct X11CaptureProvider {
    session_active: bool,
}

impl X11CaptureProvider {
    pub fn new() -> Self {
        Self {
            session_active: false,
        }
    }
}

#[async_trait]
impl CaptureProvider for X11CaptureProvider {
    fn capabilities(&self) -> CaptureCapabilities {
        CaptureCapabilities {
            supports_multi_display: true,
            requires_user_consent: false,
            backend_name: "x11".to_string(),
        }
    }

    async fn start_session(&mut self) -> Result<String, String> {
        self.session_active = true;
        Ok("x11-session".to_string())
    }

    async fn capture_frame(&mut self) -> Result<CapturedFrame, String> {
        if !self.session_active {
            return Err("no active session".to_string());
        }

        let monitors = xcap::Monitor::all()
            .map_err(|e| format!("enumerate monitors: {}", e))?;

        let monitor = monitors.first()
            .ok_or("no monitors found")?;

        let image = monitor.capture_image()
            .map_err(|e| format!("capture: {}", e))?;

        // Resize to max 1600px longest side
        let (width, height) = image.dimensions();
        let max_dim = 1600;
        let scale = if width > height {
            max_dim as f32 / width as f32
        } else {
            max_dim as f32 / height as f32
        };

        let resized = if scale < 1.0 {
            image::imageops::resize(
                &image,
                (width as f32 * scale) as u32,
                (height as f32 * scale) as u32,
                image::imageops::FilterType::Triangle,
            )
        } else {
            image
        };

        // Encode as JPEG
        let mut jpeg_buf = Vec::new();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, 75);
        resized.write_with_encoder(encoder)
            .map_err(|e| format!("encode jpeg: {}", e))?;

        Ok(CapturedFrame {
            data: jpeg_buf,
            width: resized.width(),
            height: resized.height(),
            display_id: format!("x11-{}", monitor.name().unwrap_or("unknown")),
            captured_at: chrono::Utc::now(),
        })
    }

    async fn stop_session(&mut self) -> Result<(), String> {
        self.session_active = false;
        Ok(())
    }
}
```

Update `src-tauri/src/capture/mod.rs`:
```rust
pub mod fake;
pub mod x11;

pub use fake::FakeCaptureProvider;
pub use x11::X11CaptureProvider;
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/capture/ src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add X11 capture provider with image resize and JPEG encoding"
```

---

### Task 21: System Tray Integration

**Covers:** §8 (tray menu)

**Files:**
- Create: `src-tauri/src/platform/tray.rs`
- Create: `src-tauri/src/platform/notifications.rs`

- [ ] **Step 1: Write tray module**

Create `src-tauri/src/platform/tray.rs`:
```rust
use tauri::{
    AppHandle, Manager,
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState, Menu, MenuItem},
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::new()
        .add_item(MenuItem::with_id(
            app,
            "toggle",
            "Start Recording",
            true,
            None::<&str>,
        )?)
        .add_native_item(tauri::menu::NativeMenuItem::Separator)
        .add_item(MenuItem::with_id(
            app,
            "open",
            "Open Daytrace",
            true,
            None::<&str>,
        )?)
        .add_item(MenuItem::with_id(
            app,
            "quit",
            "Quit",
            true,
            None::<&str>,
        )?)?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                // Toggle recording state
                if let Some(scheduler) = app.try_state::<std::sync::Arc<crate::pipeline::scheduler::CaptureScheduler>>() {
                    match scheduler.state() {
                        crate::pipeline::scheduler::RecordingState::Recording => {
                            scheduler.pause().ok();
                        }
                        _ => {
                            scheduler.start().ok();
                        }
                    }
                }
            }
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

Create `src-tauri/src/platform/notifications.rs`:
```rust
pub fn notify(title: &str, body: &str) {
    notify_rust::Notification::new()
        .appname("Daytrace")
        .summary(title)
        .body(body)
        .show()
        .ok();
}

pub fn notify_recording_started() {
    notify("Daytrace", "Recording started");
}

pub fn notify_recording_paused() {
    notify("Daytrace", "Recording paused");
}

pub fn notify_budget_exceeded(spent: f64, budget: f64) {
    notify(
        "Daytrace - Budget Exceeded",
        &format!("Daily budget exceeded: ${:.2} / ${:.2}", spent, budget),
    );
}

pub fn notify_analysis_failed(error: &str) {
    notify("Daytrace - Analysis Failed", error);
}
```

- [ ] **Step 2: Update main.rs setup**

The tray setup needs to be added to the `main.rs` setup closure. Update the `setup` in `src-tauri/src/main.rs` to include:
```rust
// After app.manage(scheduler):
daytrace::platform::tray::create_tray(app.handle())?;
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/platform/tray.rs src-tauri/src/platform/notifications.rs src-tauri/src/main.rs
git commit -m "feat: add system tray with start/pause/open/quit menu"
```

---

### Task 22: Settings Page

**Covers:** §8 (settings page)

**Files:**
- Create: `src/stores/settings.ts`
- Modify: `src/pages/Settings.tsx`

- [ ] **Step 1: Write settings store**

Create `src/stores/settings.ts`:
```typescript
import { create } from 'zustand';
import type { AppSettings } from '../lib/types';
import { DEFAULT_SETTINGS } from '../lib/constants';

interface SettingsStore {
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;
  updateSettings: (partial: Partial<AppSettings>) => void;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: DEFAULT_SETTINGS,
  setSettings: (settings) => set({ settings }),
  updateSettings: (partial) =>
    set((state) => ({
      settings: { ...state.settings, ...partial },
    })),
}));
```

- [ ] **Step 2: Write Settings page**

Replace `src/pages/Settings.tsx`:
```tsx
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getSettings, saveSettings, saveProviderKey, testProviderKey, clearAllData } from '../api/tauri';
import { PROVIDERS } from '../lib/constants';
import type { AppSettings } from '../lib/types';
import { useState } from 'react';

export function Settings() {
  const queryClient = useQueryClient();
  const { data: settings, isLoading } = useQuery({
    queryKey: ['settings'],
    queryFn: getSettings,
  });

  const saveMutation = useMutation({
    mutationFn: saveSettings,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['settings'] }),
  });

  const clearMutation = useMutation({
    mutationFn: clearAllData,
    onSuccess: () => queryClient.invalidateQueries(),
  });

  const [apiKey, setApiKey] = useState('');
  const [keyStatus, setKeyStatus] = useState<boolean | null>(null);

  if (isLoading || !settings) return <div>Loading...</div>;

  const handleSave = (partial: Partial<AppSettings>) => {
    saveMutation.mutate({ ...settings, ...partial });
  };

  const handleSaveKey = async () => {
    if (!apiKey.trim()) return;
    await saveProviderKey(settings.provider.vision_provider, apiKey.trim());
    setApiKey('');
    setKeyStatus(true);
  };

  const handleTestKey = async () => {
    const hasKey = await testProviderKey(settings.provider.vision_provider);
    setKeyStatus(hasKey);
  };

  return (
    <div className="max-w-2xl space-y-8">
      <h2 className="text-lg font-semibold">Settings</h2>

      <section className="space-y-4">
        <h3 className="font-medium">API Provider</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm text-gray-600 mb-1">Vision Provider</label>
            <select
              value={settings.provider.vision_provider}
              onChange={(e) =>
                handleSave({
                  provider: { ...settings.provider, vision_provider: e.target.value },
                })
              }
              className="w-full border rounded px-3 py-2"
            >
              {PROVIDERS.map((p) => (
                <option key={p.id} value={p.id}>{p.label}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm text-gray-600 mb-1">Vision Model</label>
            <input
              type="text"
              value={settings.provider.vision_model}
              onChange={(e) =>
                handleSave({
                  provider: { ...settings.provider, vision_model: e.target.value },
                })
              }
              className="w-full border rounded px-3 py-2"
            />
          </div>
        </div>

        <div className="flex gap-2">
          <input
            type="password"
            placeholder="API Key"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            className="flex-1 border rounded px-3 py-2"
          />
          <button onClick={handleSaveKey} className="px-4 py-2 bg-blue-500 text-white rounded">
            Save Key
          </button>
          <button onClick={handleTestKey} className="px-4 py-2 border rounded">
            Test Key
          </button>
        </div>
        {keyStatus !== null && (
          <p className={keyStatus ? 'text-green-600' : 'text-red-600'}>
            {keyStatus ? 'Key is saved' : 'No key found'}
          </p>
        )}
      </section>

      <section className="space-y-4">
        <h3 className="font-medium">Capture</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm text-gray-600 mb-1">Interval (seconds)</label>
            <input
              type="number"
              value={settings.capture_interval_sec}
              onChange={(e) =>
                handleSave({ capture_interval_sec: parseInt(e.target.value) || 300 })
              }
              min={60}
              max={3600}
              className="w-full border rounded px-3 py-2"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-600 mb-1">Idle threshold (seconds)</label>
            <input
              type="number"
              value={settings.idle_threshold_sec}
              onChange={(e) =>
                handleSave({ idle_threshold_sec: parseInt(e.target.value) || 300 })
              }
              min={60}
              max={3600}
              className="w-full border rounded px-3 py-2"
            />
          </div>
        </div>
      </section>

      <section className="space-y-4">
        <h3 className="font-medium">Budget</h3>
        <div>
          <label className="block text-sm text-gray-600 mb-1">Daily budget (cents)</label>
          <input
            type="number"
            value={settings.daily_budget_cents}
            onChange={(e) =>
              handleSave({ daily_budget_cents: parseInt(e.target.value) || 500 })
            }
            min={0}
            className="w-full border rounded px-3 py-2"
          />
        </div>
      </section>

      <section className="space-y-4">
        <h3 className="font-medium text-red-600">Danger Zone</h3>
        <button
          onClick={() => {
            if (confirm('Delete all local data? This cannot be undone.')) {
              clearMutation.mutate();
            }
          }}
          className="px-4 py-2 bg-red-500 text-white rounded"
        >
          Clear All Data
        </button>
      </section>
    </div>
  );
}
```

- [ ] **Step 3: Verify frontend builds**

Run:
```bash
npm run build
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/stores/settings.ts src/pages/Settings.tsx
git commit -m "feat: add settings page with provider config and data management"
```

---

### Task 23: Reports Page

**Covers:** §8 (report generation, editing, export)

**Files:**
- Create: `src/components/ReportEditor.tsx`
- Modify: `src/pages/Reports.tsx`

- [ ] **Step 1: Write Reports page**

Replace `src/pages/Reports.tsx`:
```tsx
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listReports, generateReport } from '../api/tauri';
import { TEMPLATES } from '../lib/constants';
import type { PeriodType } from '../lib/types';

export function Reports() {
  const queryClient = useQueryClient();
  const [periodType, setPeriodType] = useState<PeriodType>('daily');
  const [periodStart, setPeriodStart] = useState(
    new Date().toISOString().split('T')[0]
  );
  const [templateId, setTemplateId] = useState('standard');
  const [selectedReport, setSelectedReport] = useState<string | null>(null);

  const { data: reports = [] } = useQuery({
    queryKey: ['reports', periodType],
    queryFn: () => listReports(periodType),
  });

  const generateMutation = useMutation({
    mutationFn: () => generateReport(periodType, periodStart, templateId),
    onSuccess: (report) => {
      queryClient.invalidateQueries({ queryKey: ['reports'] });
      setSelectedReport(report.id);
    },
  });

  const activeReport = reports.find((r) => r.id === selectedReport);

  return (
    <div className="flex gap-6 h-full">
      <div className="w-80 flex-shrink-0 space-y-4">
        <h2 className="text-lg font-semibold">Generate Report</h2>

        <div>
          <label className="block text-sm text-gray-600 mb-1">Period</label>
          <select
            value={periodType}
            onChange={(e) => setPeriodType(e.target.value as PeriodType)}
            className="w-full border rounded px-3 py-2"
          >
            <option value="daily">Daily</option>
            <option value="weekly">Weekly</option>
            <option value="monthly">Monthly</option>
          </select>
        </div>

        <div>
          <label className="block text-sm text-gray-600 mb-1">Start Date</label>
          <input
            type="date"
            value={periodStart}
            onChange={(e) => setPeriodStart(e.target.value)}
            className="w-full border rounded px-3 py-2"
          />
        </div>

        <div>
          <label className="block text-sm text-gray-600 mb-1">Template</label>
          <select
            value={templateId}
            onChange={(e) => setTemplateId(e.target.value)}
            className="w-full border rounded px-3 py-2"
          >
            {TEMPLATES.map((t) => (
              <option key={t.id} value={t.id}>{t.label}</option>
            ))}
          </select>
        </div>

        <button
          onClick={() => generateMutation.mutate()}
          disabled={generateMutation.isPending}
          className="w-full px-4 py-2 bg-blue-500 text-white rounded disabled:opacity-50"
        >
          {generateMutation.isPending ? 'Generating...' : 'Generate Report'}
        </button>

        {generateMutation.isError && (
          <p className="text-red-600 text-sm">{String(generateMutation.error)}</p>
        )}

        <hr />

        <h3 className="font-medium">History</h3>
        <div className="space-y-1 max-h-64 overflow-auto">
          {reports.map((report) => (
            <button
              key={report.id}
              onClick={() => setSelectedReport(report.id)}
              className={`w-full text-left px-3 py-2 rounded text-sm ${
                selectedReport === report.id
                  ? 'bg-blue-100 text-blue-700'
                  : 'hover:bg-gray-100'
              }`}
            >
              {report.title}
            </button>
          ))}
          {reports.length === 0 && (
            <p className="text-gray-500 text-sm">No reports yet</p>
          )}
        </div>
      </div>

      <div className="flex-1">
        {activeReport ? (
          <div>
            <div className="flex justify-between items-center mb-4">
              <h3 className="font-medium">{activeReport.title}</h3>
              <div className="flex gap-2">
                <button
                  onClick={() => navigator.clipboard.writeText(activeReport.content_markdown)}
                  className="px-3 py-1 text-sm border rounded hover:bg-gray-50"
                >
                  Copy
                </button>
                <button
                  onClick={() => {
                    const blob = new Blob([activeReport.content_markdown], {
                      type: 'text/markdown',
                    });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = `${activeReport.title}.md`;
                    a.click();
                    URL.revokeObjectURL(url);
                  }}
                  className="px-3 py-1 text-sm border rounded hover:bg-gray-50"
                >
                  Export .md
                </button>
              </div>
            </div>
            <div className="prose max-w-none bg-white p-6 rounded border">
              <pre className="whitespace-pre-wrap text-sm">
                {activeReport.content_markdown}
              </pre>
            </div>
          </div>
        ) : (
          <div className="text-gray-500 text-center py-12">
            Generate or select a report to view
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify frontend builds**

Run:
```bash
npm run build
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/pages/Reports.tsx
git commit -m "feat: add reports page with generation, history, and export"
```

---

## Phase 9: Final Verification

### Task 24: Full Build and Test Verification

**Covers:** §11 (verification)

- [ ] **Step 1: Run Rust tests**

Run:
```bash
cd src-tauri && cargo test
```

Expected: All tests pass (20+ tests).

- [ ] **Step 2: Run Rust clippy**

Run:
```bash
cd src-tauri && cargo clippy -- -D warnings
```

Expected: No warnings.

- [ ] **Step 3: Run Rust fmt check**

Run:
```bash
cd src-tauri && cargo fmt --check
```

Expected: No formatting issues.

- [ ] **Step 4: Run frontend build**

Run:
```bash
npm run build
```

Expected: No TypeScript errors, Vite builds successfully.

- [ ] **Step 5: Verify full Tauri build**

Run:
```bash
npm run tauri build 2>&1 | head -50
```

Expected: Build starts without errors. Full build may take a while.

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "chore: final verification - all tests pass, clean build"
```

---

## Verification Checklist

After completing all tasks, verify:

- [ ] `cargo test` passes all tests
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt --check` is clean
- [ ] `npm run build` succeeds
- [ ] Recording state machine works (start/pause/stop)
- [ ] Fake capture provider generates frames
- [ ] Dedup detection works for identical frames
- [ ] Activity CRUD operations work
- [ ] Job queue claim/complete/fail works
- [ ] Settings save/load round-trips
- [ ] Report generation produces markdown
- [ ] Tauri commands are registered and callable
- [ ] Frontend renders without errors
- [ ] API key stored in system keyring (not database)
- [ ] Temp files created with correct permissions
- [ ] Cleanup removes temp files on startup
