# Patent Draft Pro — MVP Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an AI-powered patent drafting SaaS for Indian patent agents — guided interview → AI-generated draft → section editor → PDF/DOCX export.

**Architecture:** Rust (Axum) backend with provider-agnostic AI pipeline and SSE streaming, Next.js 16 frontend with section-based card editor, Supabase Postgres (DB only), Cloudflare R2 (file storage), Razorpay (billing). Self-managed auth (argon2 + JWT). All webhooks in Rust. Monorepo with Cargo workspace (backend) + npm workspace (frontend).

**Tech Stack:** Rust (Axum, sqlx, tokio, reqwest, argon2, jsonwebtoken, aws-sdk-s3), Next.js 16 (App Router), React, shadcn/ui, Tailwind CSS, Razorpay, Playwright, Vitest, typst CLI, docx-rs

**Spec:** `docs/superpowers/specs/2026-03-18-patent-draft-pro-mvp-design.md`

**New Repo:** This plan is for a new repository `patent-draft-pro` (NOT the dasari-ai repo).

---

## File Structure

```
patent-draft-pro/
├── apps/
│   └── web/                              ← Next.js 16 (App Router)
│       ├── app/
│       │   ├── layout.tsx                ← Root layout (Geist fonts, dark mode)
│       │   ├── login/page.tsx            ← Login form (calls Rust /api/auth/login)
│       │   ├── subscribe/page.tsx        ← Subscription gate (Razorpay checkout)
│       │   ├── projects/
│       │   │   ├── page.tsx              ← Project list dashboard
│       │   │   ├── new/page.tsx          ← Interview wizard
│       │   │   └── [id]/
│       │   │       ├── page.tsx          ← Section-based editor
│       │   │       └── export/page.tsx   ← Export page
│       │   └── account/page.tsx          ← Profile + billing
│       ├── components/
│       │   ├── ui/                       ← shadcn/ui primitives
│       │   ├── interview/
│       │   │   ├── interview-wizard.tsx  ← Wizard container + step routing
│       │   │   ├── step-basics.tsx       ← Step 1: title, type, field
│       │   │   ├── step-applicant.tsx   ← Step 2: applicant/inventor/agent details
│       │   │   ├── step-problem.tsx      ← Step 3: problem, prior art
│       │   │   ├── step-description.tsx  ← Step 4: invention description
│       │   │   ├── step-novelty.tsx      ← Step 5: novelty, advantages
│       │   │   ├── step-figures.tsx      ← Step 6: figure uploads
│       │   │   └── step-review.tsx       ← Step 7: review + generate
│       │   ├── editor/
│       │   │   ├── section-card.tsx      ← Single section card (view/edit/regenerate/history)
│       │   │   ├── section-list.tsx      ← Ordered list of all sections
│       │   │   ├── version-history.tsx   ← Version history panel for a section
│       │   │   └── generation-stream.tsx ← SSE stream display during generation
│       │   ├── projects/
│       │   │   └── project-card.tsx      ← Project list item
│       │   └── layout/
│       │       ├── navbar.tsx            ← Top nav
│       │       ├── auth-guard.tsx        ← Redirect to /login if unauthenticated
│       │       └── subscription-guard.tsx← Redirect to /subscribe if no active sub
│       ├── hooks/
│       │   ├── use-interview-wizard.ts   ← Wizard state + persistence
│       │   ├── use-sse-stream.ts         ← SSE event parser hook
│       │   ├── use-auto-save.ts          ← Debounced auto-save
│       │   └── use-auth.ts              ← Auth state (JWT from cookie, refresh)
│       ├── lib/
│       │   ├── api-client.ts             ← Typed HTTP client for Rust backend
│       │   └── razorpay.ts              ← Razorpay checkout helpers
│       ├── next.config.ts
│       ├── tailwind.config.ts
│       ├── tsconfig.json
│       ├── vitest.config.ts
│       └── package.json
│
├── backend/
│   ├── Cargo.toml                        ← Workspace root
│   ├── api/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                   ← Axum server setup, router, startup
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── health.rs             ← GET /api/health
│   │       │   ├── auth.rs              ← POST /api/auth/login, /refresh, /logout
│   │       │   ├── me.rs                ← GET /api/me
│   │       │   ├── projects.rs           ← CRUD /api/projects
│   │       │   ├── interview.rs          ← GET/PUT /api/projects/:id/interview
│   │       │   ├── figures.rs            ← POST/GET/DELETE /api/projects/:id/figures
│   │       │   ├── generate.rs           ← POST /api/projects/:id/generate (SSE)
│   │       │   ├── sections.rs           ← PUT/POST sections + GET/POST versions
│   │       │   ├── export.rs             ← POST/GET /api/projects/:id/export
│   │       │   ├── subscriptions.rs     ← POST /api/subscriptions/create
│   │       │   └── webhooks.rs          ← POST /api/webhooks/razorpay
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs               ← JWT validation + issuance
│   │       │   ├── subscription.rs       ← Subscription check
│   │       │   └── rate_limit.rs        ← Per-user rate limiting
│   │       └── error.rs                  ← Unified error handling
│   ├── ai/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provider.rs               ← LlmProvider trait + factory
│   │       ├── anthropic.rs              ← Claude implementation
│   │       ├── mock.rs                   ← Mock provider for tests
│   │       ├── pipeline.rs               ← Multi-section generation orchestrator
│   │       └── prompts/
│   │           ├── mod.rs                ← Prompt builder
│   │           ├── title.txt
│   │           ├── field_of_invention.txt
│   │           ├── background.txt
│   │           ├── summary.txt
│   │           ├── detailed_description.txt
│   │           ├── claims.txt
│   │           ├── abstract.txt
│   │           ├── drawings_description.txt
│   │           └── cross_reference.txt
│   ├── export/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pdf.rs                    ← typst template + CLI invocation
│   │       ├── docx.rs                   ← docx-rs generation
│   │       └── templates/
│   │           └── ipo_form2.typ         ← typst template for IPO Form 2
│   ├── storage/                          ← Storage client crate (local + R2)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs                 ← StorageClient trait + factory
│   │       ├── local.rs                  ← LocalStorage (filesystem, for dev)
│   │       └── r2.rs                     ← R2Storage (S3-compatible, for prod)
│   └── shared/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── models.rs                 ← DB models (sqlx FromRow)
│           ├── db.rs                     ← Connection pool setup
│           └── config.rs                 ← Environment config
│
├── migrations/
│   ├── 001_create_users.sql
│   ├── 002_create_sessions.sql
│   ├── 003_create_projects.sql
│   ├── 004_create_project_applicants.sql
│   ├── 005_create_interview_responses.sql
│   ├── 006_create_patent_sections.sql
│   ├── 007_create_section_versions.sql
│   ├── 008_create_figures.sql
│   ├── 009_create_exports.sql
│   ├── 010_create_subscriptions.sql
│   └── 011_create_rate_limits.sql
│
├── tests/                                ← Rust integration tests
│   ├── common/
│   │   └── mod.rs                        ← Test helpers (setup DB, create test user, mock JWT)
│   ├── auth_test.rs
│   ├── projects_test.rs
│   ├── interview_test.rs
│   ├── figures_test.rs
│   ├── generate_test.rs
│   ├── sections_test.rs
│   ├── versions_test.rs
│   ├── export_test.rs
│   ├── webhooks_test.rs
│   └── rate_limit_test.rs
│
├── e2e/                                  ← Playwright E2E tests
│   ├── playwright.config.ts
│   ├── fixtures/
│   │   └── test-setup.ts                 ← Auth fixtures, seed data
│   ├── auth.spec.ts
│   ├── interview.spec.ts
│   ├── generation.spec.ts
│   ├── editor.spec.ts
│   ├── export.spec.ts
│   ├── subscription.spec.ts
│   └── projects.spec.ts
│
├── .github/
│   └── workflows/
│       ├── rust.yml                      ← Rust unit + integration tests
│       ├── frontend.yml                  ← Frontend unit + integration tests
│       └── e2e.yml                       ← E2E tests
│
├── docker-compose.yml                    ← Local dev: Postgres
├── Dockerfile.backend                    ← Rust backend container for Fly.io
├── fly.toml                              ← Fly.io config
├── Makefile                              ← Dev commands
├── .env.example
├── .gitignore
└── README.md
```

---

## Phase 1: Project Scaffold & Infrastructure (Tasks 1-3)

### Task 1: Create repo and monorepo scaffold

**Files:**

- Create: `patent-draft-pro/` (new repo root)
- Create: `Makefile`
- Create: `.gitignore`
- Create: `.env.example`
- Create: `docker-compose.yml`
- Create: `backend/Cargo.toml` (workspace)
- Create: `backend/shared/Cargo.toml`
- Create: `backend/shared/src/lib.rs`
- Create: `backend/shared/src/config.rs`
- Create: `backend/api/Cargo.toml`
- Create: `backend/api/src/main.rs`
- Create: `backend/ai/Cargo.toml`
- Create: `backend/ai/src/lib.rs`
- Create: `backend/export/Cargo.toml`
- Create: `backend/export/src/lib.rs`

- [ ] **Step 1: Create new repo and initialize git**

```bash
mkdir patent-draft-pro && cd patent-draft-pro
git init
```

- [ ] **Step 2: Create .gitignore**

```gitignore
# Rust
target/
*.swp

# Node
node_modules/
.next/
out/

# Environment
.env
.env.local
.env*.local

# Local file storage (dev)
storage/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
```

- [ ] **Step 3: Create .env.example**

```env
# Backend
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/patent_draft_pro
JWT_SECRET=change-me-64-chars-minimum-for-hmac-sha256-security
ANTHROPIC_API_KEY=sk-ant-xxx       # optional when AI_PROVIDER=mock
OPENAI_API_KEY=sk-xxx              # optional, for future use
AI_PROVIDER=mock                   # "anthropic" | "mock" (use mock for local dev)
RUST_LOG=info
PORT=5012
ALLOWED_ORIGIN=http://localhost:3000   # Lock CORS to frontend origin (use production URL in prod)

# Storage: "local" for dev (no R2 credentials needed), "r2" for production
STORAGE_BACKEND=local
STORAGE_LOCAL_PATH=./storage       # local dev: files saved here (auto-created)

# Cloudflare R2 (only needed when STORAGE_BACKEND=r2)
# R2_ACCOUNT_ID=your-cf-account-id
# R2_ACCESS_KEY_ID=your-r2-access-key
# R2_SECRET_ACCESS_KEY=your-r2-secret-key
# R2_BUCKET_NAME=patent-draft-pro
# R2_PUBLIC_URL=https://files.patentdraftpro.com

# Razorpay (test credentials for local dev)
RAZORPAY_KEY_ID=rzp_test_xxx
RAZORPAY_KEY_SECRET=your-razorpay-secret
RAZORPAY_WEBHOOK_SECRET=your-webhook-secret
RAZORPAY_PLAN_ID=plan_xxx

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:5012
NEXT_PUBLIC_RAZORPAY_KEY_ID=rzp_test_xxx
```

- [ ] **Step 4: Create docker-compose.yml**

```yaml
version: "3.8"
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: patent_draft_pro
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

- [ ] **Step 5: Create Rust workspace Cargo.toml**

```toml
# backend/Cargo.toml
[workspace]
members = ["api", "ai", "export", "storage", "shared"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[workspace.dev-dependencies]
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
```

> **Note:** Integration tests (in `backend/tests/`) use `testcontainers` to spin up an ephemeral Postgres instance per test suite. No manual Docker setup is needed for local test runs. In CI, Postgres is provisioned as a GitHub Actions service instead.

- [ ] **Step 6: Create shared crate**

```toml
# backend/shared/Cargo.toml
[package]
name = "shared"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx.workspace = true
serde.workspace = true
uuid.workspace = true
chrono.workspace = true
anyhow.workspace = true
```

```rust
// backend/shared/src/lib.rs
pub mod config;
pub mod db;
pub mod models;
```

```rust
// backend/shared/src/config.rs
use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub port: u16,
    pub ai_provider: String,
    // Storage
    pub storage_backend: String,           // "local" or "r2"
    pub storage_local_path: Option<String>, // only for local backend
    // Cloudflare R2 (only required when storage_backend = "r2")
    pub r2_account_id: Option<String>,
    pub r2_access_key_id: Option<String>,
    pub r2_secret_access_key: Option<String>,
    pub r2_bucket_name: Option<String>,
    pub r2_public_url: Option<String>,
    // Razorpay
    pub razorpay_key_id: String,
    pub razorpay_key_secret: String,
    pub razorpay_webhook_secret: String,
    pub razorpay_plan_id: String,
    // CORS
    pub allowed_origin: String,  // Lock to frontend URL in production
}

impl AppConfig {
    /// Build config from a map (testable without env mutation).
    pub fn from_map(vars: &std::collections::HashMap<String, String>) -> Result<Self> {
        let get = |key: &str| -> Option<String> { vars.get(key).cloned() };
        let require = |key: &str| -> Result<String> {
            get(key).context(format!("{key} must be set"))
        };

        let ai_provider = get("AI_PROVIDER")
            .unwrap_or_else(|| "anthropic".to_string());

        Ok(Self {
            database_url: require("DATABASE_URL")?,
            jwt_secret: require("JWT_SECRET")?,
            anthropic_api_key: get("ANTHROPIC_API_KEY"),
            openai_api_key: get("OPENAI_API_KEY"),
            port: get("PORT")
                .unwrap_or_else(|| "5012".to_string())
                .parse()
                .context("PORT must be a valid u16")?,
            ai_provider,
            storage_backend: get("STORAGE_BACKEND")
                .unwrap_or_else(|| "local".to_string()),
            storage_local_path: get("STORAGE_LOCAL_PATH"),
            // R2 fields are optional — only validated at runtime when storage_backend=r2
            r2_account_id: get("R2_ACCOUNT_ID"),
            r2_access_key_id: get("R2_ACCESS_KEY_ID"),
            r2_secret_access_key: get("R2_SECRET_ACCESS_KEY"),
            r2_bucket_name: get("R2_BUCKET_NAME"),
            r2_public_url: get("R2_PUBLIC_URL"),
            razorpay_key_id: require("RAZORPAY_KEY_ID")?,
            razorpay_key_secret: require("RAZORPAY_KEY_SECRET")?,
            razorpay_webhook_secret: require("RAZORPAY_WEBHOOK_SECRET")?,
            razorpay_plan_id: require("RAZORPAY_PLAN_ID")?,
            allowed_origin: get("ALLOWED_ORIGIN")
                .unwrap_or_else(|| "http://localhost:3000".to_string()),
        })
    }

    /// Convenience: read from real environment variables (calls from_map internally).
    pub fn from_env() -> Result<Self> {
        let vars: std::collections::HashMap<String, String> = std::env::vars().collect();
        Self::from_map(&vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn required_vars() -> HashMap<String, String> {
        HashMap::from([
            ("DATABASE_URL".into(), "postgresql://test".into()),
            ("JWT_SECRET".into(), "test-secret-64-chars-long-enough-for-hmac".into()),
            ("STORAGE_BACKEND".into(), "local".into()),
            ("RAZORPAY_KEY_ID".into(), "rzp_test".into()),
            ("RAZORPAY_KEY_SECRET".into(), "test".into()),
            ("RAZORPAY_WEBHOOK_SECRET".into(), "test".into()),
            ("RAZORPAY_PLAN_ID".into(), "plan_test".into()),
        ])
    }

    #[test]
    fn test_default_port() {
        // PORT not in map → defaults to 5012
        let config = AppConfig::from_map(&required_vars()).unwrap();
        assert_eq!(config.port, 5012);
    }

    #[test]
    fn test_default_ai_provider() {
        // AI_PROVIDER not in map → defaults to "anthropic"
        let config = AppConfig::from_map(&required_vars()).unwrap();
        assert_eq!(config.ai_provider, "anthropic");
    }

    #[test]
    fn test_anthropic_key_optional_with_mock() {
        let mut vars = required_vars();
        vars.insert("AI_PROVIDER".into(), "mock".into());
        // ANTHROPIC_API_KEY not in map
        let config = AppConfig::from_map(&vars).unwrap();
        assert!(config.anthropic_api_key.is_none());
    }
}
```

- [ ] **Step 7: Create api crate with minimal Axum server**

```toml
# backend/api/Cargo.toml
[package]
name = "api"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared" }
storage = { path = "../storage" }
axum = "0.8"
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
sqlx.workspace = true
uuid.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
anyhow.workspace = true
tower-http = { version = "0.6", features = ["cors", "trace"] }
http = "1"  # needed for CorsLayer configuration (HeaderValue, Method, header constants)
```

```rust
// backend/api/src/main.rs
use axum::{routing::get, Json, Router};
use shared::config::AppConfig;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env()?;

    let cors = CorsLayer::new()
        .allow_origin(config.allowed_origin.parse::<http::HeaderValue>().expect("valid ALLOWED_ORIGIN"))
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::PUT, http::Method::PATCH, http::Method::DELETE])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .allow_credentials(true);

    let app = Router::new()
        .route("/api/health", get(health))
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

- [ ] **Step 8: Create stub ai, export, and storage crates**

```toml
# backend/ai/Cargo.toml
[package]
name = "ai"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared" }
serde.workspace = true
anyhow.workspace = true
tokio.workspace = true
```

```rust
// backend/ai/src/lib.rs
pub mod provider;
```

```toml
# backend/storage/Cargo.toml
[package]
name = "storage"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared" }
aws-sdk-s3 = "1"
aws-config = { version = "1", features = ["behavior-version-latest"] }
aws-credential-types = "1"
tokio.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

```rust
// backend/storage/src/lib.rs
pub mod r2;
pub mod local;
pub mod client;
```

```rust
// backend/storage/src/client.rs
// StorageClient trait — abstraction over R2 and local filesystem
//
// trait StorageClient: Send + Sync {
//     async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> Result<String>;
//     async fn download_url(&self, key: &str, expiry_secs: u64) -> Result<String>;
//     async fn delete(&self, key: &str) -> Result<()>;
// }
//
// Factory: create_storage_client(config) → Box<dyn StorageClient>
//   - config.storage_backend == "local" → LocalStorage
//   - config.storage_backend == "r2" → R2Storage (validates R2 credentials are present)
```

```rust
// backend/storage/src/r2.rs
// R2Storage — Cloudflare R2 client (S3-compatible)
// Implements StorageClient trait
// Uses aws-sdk-s3 with custom endpoint: https://{account_id}.r2.cloudflarestorage.com
```

```rust
// backend/storage/src/local.rs
// LocalStorage — filesystem storage for development
// Implements StorageClient trait
// - upload: writes to {base_path}/{key}, creates subdirectories as needed
// - download_url: returns http://localhost:{port}/files/{key} (served by a static file route)
// - delete: removes file from disk
// Base path defaults to ./storage, configurable via STORAGE_LOCAL_PATH
```

```toml
# backend/export/Cargo.toml
[package]
name = "export"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared" }
serde.workspace = true
anyhow.workspace = true
```

```rust
// backend/export/src/lib.rs
// Export service - PDF and DOCX generation
```

- [ ] **Step 9: Create Makefile**

> **Prerequisite:** `sqlx-cli` must be installed: `cargo install sqlx-cli --no-default-features --features postgres`. This is needed for `make migrate`.

```makefile
.PHONY: dev dev-backend dev-frontend db-up db-down migrate seed-user test test-backend test-frontend test-e2e setup-tools

setup-tools:
	cargo install sqlx-cli --no-default-features --features postgres

db-up:
	docker-compose up -d

db-down:
	docker-compose down

migrate: db-up
	cd backend && sqlx migrate run --source ../migrations

seed-user: migrate
	@echo "Creating test user with active subscription (email: test@example.com, password: testpass123)"
	cd backend && cargo run -p api -- seed-user --email test@example.com --password testpass123 --name "Test User" --with-subscription

dev-backend: db-up
	cd backend && cargo run -p api

dev-frontend:
	cd apps/web && npm run dev

dev:
	make dev-backend & make dev-frontend

test-backend:
	cd backend && cargo test --workspace

test-frontend:
	cd apps/web && npm run test

test-e2e:
	npx playwright test

test: test-backend test-frontend

clean:
	docker-compose down -v
	rm -rf storage/
```

- [ ] **Step 10: Verify Rust workspace compiles**

Run: `cd backend && cargo build 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 11: Verify health endpoint**

Run: `cd backend && cargo run -p api &` then `curl http://localhost:5012/api/health`
Expected: `{"status":"ok"}`

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "feat: scaffold monorepo with Rust workspace and Axum health endpoint"
```

---

### Task 2: Database migrations and models

**Files:**

- Create: `migrations/001_create_users.sql`
- Create: `migrations/002_create_sessions.sql`
- Create: `migrations/003_create_projects.sql`
- Create: `migrations/004_create_project_applicants.sql`
- Create: `migrations/005_create_interview_responses.sql`
- Create: `migrations/006_create_patent_sections.sql`
- Create: `migrations/007_create_section_versions.sql`
- Create: `migrations/008_create_figures.sql`
- Create: `migrations/009_create_exports.sql`
- Create: `migrations/010_create_subscriptions.sql`
- Create: `migrations/011_create_rate_limits.sql`
- Create: `backend/shared/src/models.rs`
- Create: `backend/shared/src/db.rs`

- [ ] **Step 1: Create migration files**

```sql
-- migrations/001_create_users.sql

-- Shared trigger function for auto-updating updated_at on all tables
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT UNIQUE NOT NULL,
    password_hash   TEXT NOT NULL,
    full_name       TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
```

> **Note:** Every subsequent migration that creates a table with `updated_at` should also add a trigger: `CREATE TRIGGER trg_{table}_updated_at BEFORE UPDATE ON {table} FOR EACH ROW EXECUTE FUNCTION set_updated_at();`. This ensures the application never needs to manually set `updated_at`.

```sql
-- migrations/002_create_sessions.sql
CREATE TABLE sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked         BOOLEAN NOT NULL DEFAULT false,
    user_agent      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_refresh_token_hash ON sessions(refresh_token_hash) WHERE NOT revoked;
```

```sql
-- migrations/003_create_projects.sql
CREATE TABLE projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'draft',
    jurisdiction    TEXT NOT NULL DEFAULT 'IPO',
    patent_type     TEXT NOT NULL DEFAULT 'complete',
    deleted_at      TIMESTAMPTZ,  -- NULL = active, set = soft-deleted
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_projects_user_id ON projects(user_id) WHERE deleted_at IS NULL;
```

```sql
-- migrations/004_create_project_applicants.sql
CREATE TABLE project_applicants (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id            UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    applicant_name        TEXT NOT NULL,
    applicant_address     TEXT NOT NULL,
    applicant_nationality TEXT NOT NULL DEFAULT 'Indian',
    inventor_name         TEXT NOT NULL,
    inventor_address      TEXT NOT NULL,
    inventor_nationality  TEXT NOT NULL DEFAULT 'Indian',
    agent_name            TEXT,
    agent_registration_no TEXT,
    assignee_name         TEXT,
    priority_date         DATE,
    priority_country      TEXT,
    priority_application_no TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id)
);
CREATE INDEX idx_project_applicants_project_id ON project_applicants(project_id);
```

```sql
-- migrations/005_create_interview_responses.sql
CREATE TABLE interview_responses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    step_number     INT NOT NULL,
    question_key    TEXT NOT NULL,
    question_text   TEXT NOT NULL,
    response_text   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, question_key)
);
CREATE INDEX idx_interview_responses_project_id ON interview_responses(project_id);
```

```sql
-- migrations/006_create_patent_sections.sql
CREATE TABLE patent_sections (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    section_type    TEXT NOT NULL,
    content         TEXT NOT NULL,
    ai_generated    BOOLEAN NOT NULL DEFAULT true,
    edit_count      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, section_type)
);
CREATE INDEX idx_patent_sections_project_id ON patent_sections(project_id);
```

```sql
-- migrations/007_create_section_versions.sql
CREATE TABLE section_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_id      UUID NOT NULL REFERENCES patent_sections(id) ON DELETE CASCADE,
    content         TEXT NOT NULL,
    version_number  INT NOT NULL,
    source          TEXT NOT NULL DEFAULT 'manual',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(section_id, version_number)
);
CREATE INDEX idx_section_versions_section_id ON section_versions(section_id);
```

```sql
-- migrations/008_create_figures.sql
CREATE TABLE figures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    sort_order      INT NOT NULL DEFAULT 0,
    description     TEXT NOT NULL,
    storage_path    TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    content_type    TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_figures_project_id ON figures(project_id);
```

```sql
-- migrations/009_create_exports.sql
CREATE TABLE exports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    format          TEXT NOT NULL,
    storage_path    TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_exports_project_id ON exports(project_id);
```

```sql
-- migrations/010_create_subscriptions.sql
CREATE TABLE subscriptions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID NOT NULL REFERENCES users(id),
    razorpay_customer_id        TEXT NOT NULL,
    razorpay_subscription_id    TEXT NOT NULL UNIQUE,
    plan_id                     TEXT NOT NULL,
    status                      TEXT NOT NULL DEFAULT 'active',
    current_period_start        TIMESTAMPTZ NOT NULL,
    current_period_end          TIMESTAMPTZ NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_razorpay_sub_id ON subscriptions(razorpay_subscription_id);
```

```sql
-- migrations/011_create_rate_limits.sql
CREATE TABLE rate_limits (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    action_type     TEXT NOT NULL,
    window_start    TIMESTAMPTZ NOT NULL,
    request_count   INT NOT NULL DEFAULT 1,
    UNIQUE(user_id, action_type, window_start)
);
CREATE INDEX idx_rate_limits_user_action ON rate_limits(user_id, action_type, window_start);
```

- [ ] **Step 2: Create DB connection pool**

```rust
// backend/shared/src/db.rs
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}
```

- [ ] **Step 3: Create models**

```rust
// backend/shared/src/models.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]  // never send password hash to client
    pub password_hash: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub status: String,
    pub jurisdiction: String,
    pub patent_type: String,
    pub deleted_at: Option<DateTime<Utc>>,  // NULL = active, set = soft-deleted
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectApplicant {
    pub id: Uuid,
    pub project_id: Uuid,
    pub applicant_name: String,
    pub applicant_address: String,
    pub applicant_nationality: String,
    pub inventor_name: String,
    pub inventor_address: String,
    pub inventor_nationality: String,
    pub agent_name: Option<String>,
    pub agent_registration_no: Option<String>,
    pub assignee_name: Option<String>,
    pub priority_date: Option<chrono::NaiveDate>,
    pub priority_country: Option<String>,
    pub priority_application_no: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InterviewResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub step_number: i32,
    pub question_key: String,
    pub question_text: String,
    pub response_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatentSection {
    pub id: Uuid,
    pub project_id: Uuid,
    pub section_type: String,
    pub content: String,
    pub ai_generated: bool,
    pub edit_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Figure {
    pub id: Uuid,
    pub project_id: Uuid,
    pub sort_order: i32,
    pub description: String,
    pub storage_path: String,
    pub file_name: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Export {
    pub id: Uuid,
    pub project_id: Uuid,
    pub format: String,
    pub storage_path: String,
    pub file_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SectionVersion {
    pub id: Uuid,
    pub section_id: Uuid,
    pub content: String,
    pub version_number: i32,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub razorpay_customer_id: String,
    pub razorpay_subscription_id: String,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RateLimit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action_type: String,
    pub window_start: DateTime<Utc>,
    pub request_count: i32,
}

/// Valid section types for IPO Form 2
pub const SECTION_TYPES: &[&str] = &[
    "title",
    "field_of_invention",
    "background",
    "summary",
    "detailed_description",
    "claims",
    "abstract",
    "drawings_description",
];

pub fn is_valid_section_type(s: &str) -> bool {
    SECTION_TYPES.contains(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_section_types() {
        assert!(is_valid_section_type("title"));
        assert!(is_valid_section_type("claims"));
        assert!(is_valid_section_type("abstract"));
        assert!(!is_valid_section_type("invalid"));
        assert!(!is_valid_section_type(""));
    }

    #[test]
    fn test_section_types_count() {
        assert_eq!(SECTION_TYPES.len(), 8);
    }
}
```

- [ ] **Step 4: Run migrations against local Postgres**

Run: `make db-up && cd backend && sqlx migrate run --source ../migrations 2>&1 | tail -5`
Expected: All 11 migrations applied successfully

- [ ] **Step 5: Verify compilation with models**

Run: `cd backend && cargo build 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 6: Run unit tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -10`
Expected: tests pass (config + models tests)

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add database migrations and sqlx models for all tables"
```

---

### Task 3: Next.js frontend scaffold

**Files:**

- Create: `apps/web/` (via create-next-app)
- Create: `apps/web/lib/api-client.ts`
- Create: `apps/web/lib/razorpay.ts`
- Create: `apps/web/hooks/use-auth.ts`

- [ ] **Step 1: Create Next.js 16 app**

```bash
cd apps
npx create-next-app@latest web --typescript --tailwind --eslint --app --src-dir=false --import-alias="@/*"
```

- [ ] **Step 2: Install dependencies**

```bash
cd apps/web
npm install
npm install geist  # Geist Sans + Geist Mono fonts
npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom msw @vitejs/plugin-react
```

- [ ] **Step 3: Initialize shadcn/ui**

```bash
cd apps/web
npx shadcn@latest init
# Select: New York style, Zinc base color, CSS variables
npx shadcn@latest add button input textarea card label select dialog toast sheet badge tabs
```

- [ ] **Step 4: Set up Geist fonts in layout.tsx**

```tsx
// apps/web/app/layout.tsx
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import "./globals.css";

export const metadata = {
  title: "Patent Draft Pro",
  description: "AI-powered patent drafting for Indian Patent Office",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className={`${GeistSans.variable} ${GeistMono.variable} font-sans antialiased`}>
        {children}
      </body>
    </html>
  );
}
```

- [ ] **Step 5: Create auth hook**

```typescript
// apps/web/hooks/use-auth.ts
// Client-side auth state hook
// - Auth state derived from GET /api/me response (NOT by reading cookies — httpOnly cookies are inaccessible to JS)
// - Provides login(email, password), logout(), refresh() methods
// - login() calls POST /api/auth/login, backend sets httpOnly cookies via Set-Cookie header
// - Cookies are sent automatically on subsequent requests via credentials: "include"
// - isAuthenticated is determined by whether GET /api/me returns 200 vs 401
// - No Supabase dependency — pure REST calls to Rust backend
```

- [ ] **Step 6: Set up MSW test infrastructure**

Create `apps/web/test/mocks/handlers.ts` (empty handler array to start), `apps/web/test/mocks/server.ts` (MSW `setupServer` with handlers), and `apps/web/test/setup.ts` (starts MSW server before all tests, resets handlers after each, closes after all). Update `vitest.config.ts` to include `setupFiles: ['./test/setup.ts']` and set `environment: 'jsdom'`.

- [ ] **Step 7: Create typed API client**

```typescript
// apps/web/lib/api-client.ts
const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:5012";

// Auth is handled via httpOnly cookies set by the Rust backend.
// No need to manually attach Authorization headers — cookies are sent automatically.
async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, {
    ...options,
    credentials: "include", // send httpOnly cookies
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
  });

  if (res.status === 401) {
    // Try refresh, then redirect to login
    window.location.href = "/login";
    throw new Error("Unauthorized");
  }
  if (res.status === 403) {
    throw new Error("Subscription required");
  }
  if (res.status === 429) {
    throw new Error("Rate limit exceeded. Please try again later.");
  }
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.error || `Request failed: ${res.status}`);
  }

  return res.json();
}

export const api = {
  // Auth
  login: (data: { email: string; password: string }) =>
    request("/api/auth/login", { method: "POST", body: JSON.stringify(data) }),
  logout: () => request("/api/auth/logout", { method: "POST" }),
  refresh: () => request("/api/auth/refresh", { method: "POST" }),

  // User
  getMe: () => request("/api/me"),

  // Subscriptions
  createSubscription: () =>
    request("/api/subscriptions/create", { method: "POST" }),

  // Projects
  listProjects: () => request("/api/projects"),
  createProject: (data: { title: string; patent_type: string; jurisdiction?: string }) =>
    request("/api/projects", { method: "POST", body: JSON.stringify(data) }),
  getProject: (id: string) => request(`/api/projects/${id}`),
  updateProject: (id: string, data: Record<string, string>) =>
    request(`/api/projects/${id}`, { method: "PATCH", body: JSON.stringify(data) }),
  deleteProject: (id: string) => request(`/api/projects/${id}`, { method: "DELETE" }),

  // Applicant details
  getApplicant: (projectId: string) => request(`/api/projects/${projectId}/applicant`),
  upsertApplicant: (projectId: string, data: Record<string, unknown>) =>
    request(`/api/projects/${projectId}/applicant`, {
      method: "PUT",
      body: JSON.stringify(data),
    }),

  // Interview
  getInterview: (projectId: string) => request(`/api/projects/${projectId}/interview`),
  saveInterview: (projectId: string, responses: unknown[]) =>
    request(`/api/projects/${projectId}/interview`, {
      method: "PUT",
      body: JSON.stringify({ responses }),
    }),

  // Sections
  updateSection: (projectId: string, sectionType: string, content: string) =>
    request(`/api/projects/${projectId}/sections/${sectionType}`, {
      method: "PUT",
      body: JSON.stringify({ content }),
    }),

  // Section versions
  listVersions: (projectId: string, sectionType: string) =>
    request(`/api/projects/${projectId}/sections/${sectionType}/versions`),
  restoreVersion: (projectId: string, sectionType: string, versionNumber: number) =>
    request(`/api/projects/${projectId}/sections/${sectionType}/versions/${versionNumber}/restore`, {
      method: "POST",
    }),

  // Export
  createExport: (projectId: string, format: string) =>
    request(`/api/projects/${projectId}/export`, {
      method: "POST",
      body: JSON.stringify({ format }),
    }),
  listExports: (projectId: string) => request(`/api/projects/${projectId}/exports`),
  getDownloadUrl: (exportId: string) => request(`/api/exports/${exportId}/download`),
};
```

- [ ] **Step 8: Verify frontend builds**

Run: `cd apps/web && npm run build 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat: scaffold Next.js 16 frontend with shadcn/ui and API client"
```

---

## Phase 2: Auth & Middleware (Tasks 4-5)

### Task 4: Rust JWT auth middleware

**Files:**

- Create: `backend/api/src/middleware/mod.rs`
- Create: `backend/api/src/middleware/auth.rs`
- Create: `backend/api/src/middleware/subscription.rs`
- Create: `backend/api/src/middleware/rate_limit.rs`
- Create: `backend/api/src/error.rs`
- Create: `backend/api/src/routes/mod.rs`
- Create: `backend/api/src/routes/health.rs`
- Create: `backend/api/src/routes/auth.rs`
- Create: `backend/api/src/routes/me.rs`

- [ ] **Step 1: Add auth dependencies to api Cargo.toml**

Add to `backend/api/Cargo.toml`:

```toml
jsonwebtoken = "9"
argon2 = "0.5"
chrono.workspace = true
cookie = { version = "0.18", features = ["signed"] }
```

- [ ] **Step 2: Write failing test stub for JWT validation**

Create `backend/api/src/middleware/auth.rs` with the `Claims` struct undefined — just the test module importing `super::*` with `test_extract_user_id_from_valid_claims` and `test_reject_expired_token`. This will fail to compile until the implementation is written.

- [ ] **Step 3: Run test to verify it fails**

Run: `cd backend && cargo test -p api 2>&1 | tail -10`
Expected: FAIL — `Claims` not defined

- [ ] **Step 4: Implement JWT middleware (replaces Step 2's stub with full implementation + tests)**

```rust
// backend/api/src/middleware/auth.rs
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iss: String,  // "pdp:access" or "pdp:refresh" — differentiates token types
}

pub const ISS_ACCESS: &str = "pdp:access";
pub const ISS_REFRESH: &str = "pdp:refresh";

impl Claims {
    pub fn user_id(&self) -> anyhow::Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|e| anyhow::anyhow!("Invalid user ID in token: {}", e))
    }

    pub fn is_access_token(&self) -> bool {
        self.iss == ISS_ACCESS
    }

    pub fn is_refresh_token(&self) -> bool {
        self.iss == ISS_REFRESH
    }
}

pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
}

/// Issue a JWT (access token — 24h expiry)
pub fn issue_token(user_id: &Uuid, secret: &str) -> anyhow::Result<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now().timestamp() + 86400) as usize, // 24 hours
        iss: ISS_ACCESS.to_string(),
    };
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

/// Issue a refresh token (7 day expiry)
pub fn issue_refresh_token(user_id: &Uuid, secret: &str) -> anyhow::Result<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now().timestamp() + 604800) as usize, // 7 days
        iss: ISS_REFRESH.to_string(),
    };
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub async fn auth_middleware(
    State(jwt_secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try Authorization header first, then fall back to cookie
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            request.headers()
                .get("Cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';')
                        .find_map(|c| c.trim().strip_prefix("access_token="))
                        .map(|s| s.to_string())
                })
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = validate_token(&token, &jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Reject refresh tokens used as access tokens
    if !claims.is_access_token() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id = claims.user_id().map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthUser { user_id });
    Ok(next.run(request).await)
}

#[cfg(test)]
pub fn create_test_token(secret: &str, sub: &str, expires_in_secs: i64) -> String {
    create_test_token_with_iss(secret, sub, expires_in_secs, ISS_ACCESS)
}

#[cfg(test)]
pub fn create_test_token_with_iss(secret: &str, sub: &str, expires_in_secs: i64, iss: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: sub.to_string(),
        exp: (chrono::Utc::now().timestamp() + expires_in_secs) as usize,
        iss: iss.to_string(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_id_from_valid_claims() {
        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: chrono::Utc::now().timestamp() as usize + 3600,
            iss: ISS_ACCESS.to_string(),
        };
        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[test]
    fn test_reject_expired_token() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let token = create_test_token(secret, &user_id.to_string(), -3600);
        let result = validate_token(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_accept_valid_token() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let token = create_test_token(secret, &user_id.to_string(), 3600);
        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.user_id().unwrap(), user_id);
        assert!(claims.is_access_token());
    }

    #[test]
    fn test_reject_refresh_token_as_access() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let token = create_test_token_with_iss(secret, &user_id.to_string(), 3600, ISS_REFRESH);
        let claims = validate_token(&token, secret).unwrap();
        assert!(!claims.is_access_token());
        assert!(claims.is_refresh_token());
    }

    #[test]
    fn test_reject_wrong_secret() {
        let user_id = Uuid::new_v4();
        let token = create_test_token("secret-a", &user_id.to_string(), 3600);
        let result = validate_token(&token, "secret-b");
        assert!(result.is_err());
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && cargo test -p api 2>&1 | tail -10`
Expected: 4 tests pass

- [ ] **Step 6: Implement subscription middleware**

```rust
// backend/api/src/middleware/subscription.rs
use axum::{extract::State, http::StatusCode, middleware::Next, response::Response, extract::Request};
use sqlx::PgPool;
use super::auth::AuthUser;

pub async fn subscription_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_user = request
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();

    let has_active_sub = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM subscriptions WHERE user_id = $1 AND status = 'active' AND current_period_end > now())"
    )
    .bind(auth_user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_active_sub {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
```

- [ ] **Step 7: Implement /api/auth endpoints (login, refresh, logout)**

```rust
// backend/api/src/routes/auth.rs
// POST /api/auth/login
//   1. Look up user by email, verify password with argon2
//   2. Create session row in `sessions` table (store SHA-256 hash of refresh token)
//   3. Issue JWT (24h) + refresh token (7d) as httpOnly, Secure, SameSite=Lax cookies
//   4. Return { user, has_active_subscription }
//
// POST /api/auth/refresh
//   1. Read refresh_token from cookie
//   2. Decode JWT and verify iss == "pdp:refresh" (reject access tokens used as refresh tokens)
//   3. Hash it, look up in sessions table (reject if revoked or expired)
//   4. Rotate: revoke old session, create new session with new refresh token
//   5. Issue new JWT (iss: "pdp:access") + new refresh token (iss: "pdp:refresh") cookies
//
// POST /api/auth/logout
//   1. Read refresh_token from cookie
//   2. Hash it, mark session as revoked=true in DB
//   3. Clear both cookies
```

- [ ] **Step 8: Implement /api/me endpoint**

```rust
// backend/api/src/routes/me.rs
use axum::{extract::State, http::StatusCode, Json, Extension};
use serde::Serialize;
use sqlx::PgPool;
use crate::middleware::auth::AuthUser;

#[derive(Serialize)]
pub struct MeResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub has_active_subscription: bool,
}

pub async fn get_me(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
) -> Result<Json<MeResponse>, StatusCode> {
    let row = sqlx::query_as::<_, (uuid::Uuid, String, String)>(
        "SELECT id, email, full_name FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let has_active_subscription = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM subscriptions WHERE user_id = $1 AND status = 'active' AND current_period_end > now())"
    )
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    Ok(Json(MeResponse {
        id: row.0,
        email: row.1,
        full_name: row.2,
        has_active_subscription,
    }))
}

```

- [ ] **Step 9: Implement rate limiting middleware**

```rust
// backend/api/src/middleware/rate_limit.rs
// Per-user rate limiting for expensive operations (generate, regenerate, export).
// Checks rate_limits table: if request_count >= limit for the current hour window, return 429.
// Otherwise, upsert increment the counter.
// Configured limits: generate=5/hr, regenerate=20/hr, export=10/hr
```

- [ ] **Step 10: Implement `seed-user` CLI subcommand**

Add a `seed-user` subcommand to the `api` binary using `clap` (add `clap = { version = "4", features = ["derive"] }` to api/Cargo.toml). When run as `cargo run -p api -- seed-user --email test@example.com --password testpass123 --name "Test User" --with-subscription`, it:
1. Connects to the database
2. Creates a user with argon2-hashed password
3. If `--with-subscription` flag is set, inserts a `subscriptions` row with `status='active'` and `current_period_end` set 1 year in the future (razorpay fields use placeholder values like `cust_test_seed` and `sub_test_seed`)
4. Prints the created user ID

This is used by `make seed-user` for local development. No Razorpay interaction needed.

- [ ] **Step 11: Wire routes and middleware into main.rs**

Update `backend/api/src/main.rs` to compose the router:
- No auth: `/api/health`, `/api/auth/*`, `/api/webhooks/*`
- Auth only (no subscription): `/api/me`, `/api/subscriptions/*`
- Auth + subscription: all project routes
- Auth + subscription + rate limit: `/api/projects/:id/generate`, `regenerate`, `export`

- [ ] **Step 12: Run all tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 13: Commit**

```bash
git add -A
git commit -m "feat: add self-managed auth (argon2+JWT), subscription guard, rate limiting, and /api/me"
```

---

### Task 5: Frontend login page and auth guard

**Files:**

- Create: `apps/web/app/login/page.tsx`
- Create: `apps/web/components/layout/auth-guard.tsx`
- Create: `apps/web/components/layout/subscription-guard.tsx`
- Create: `apps/web/components/layout/navbar.tsx`

- [ ] **Step 1: Write login page component test**

Create `apps/web/app/login/login.test.tsx` — test renders email/password fields and submit button.

- [ ] **Step 2: Run test to verify it fails**

Run: `cd apps/web && npx vitest run login.test.tsx 2>&1 | tail -10`
Expected: FAIL

- [ ] **Step 3: Implement login page**

Build login page that calls `api.login({ email, password })`. On success, the Rust backend sets httpOnly cookies, and the frontend redirects to `/projects`. Show error on invalid credentials.

- [ ] **Step 4: Implement auth guard**

Client component that calls `GET /api/me`. If 401, redirects to `/login`. The httpOnly cookie is sent automatically via `credentials: "include"`. No Supabase dependency.

- [ ] **Step 5: Implement subscription guard**

Client component that calls `GET /api/me`, redirects to `/subscribe` if `has_active_subscription` is false.

> **Note:** Auth is cookie-based (httpOnly cookies set by Rust backend). Both auth-guard and subscription-guard use the same `GET /api/me` call — auth-guard checks for 401, subscription-guard checks the response body. For server components that need auth, forward the cookie header from the incoming request when calling the Rust backend.

- [ ] **Step 6: Implement navbar**

Simple top nav: "Patent Draft Pro" logo, "Projects" link, "Account" link, logout button.

- [ ] **Step 7: Run tests**

Run: `cd apps/web && npx vitest run 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: add login page, auth guard, subscription guard, and navbar"
```

---

## Phase 3: Core CRUD — Projects & Interview (Tasks 6-8)

### Task 6: Projects CRUD + Applicants API

**Files:**

- Create: `backend/api/src/routes/projects.rs`
- Create: `backend/api/src/routes/applicants.rs`

- [ ] **Step 1: Write integration test for create project**

Create `backend/tests/projects_test.rs` with test: POST /api/projects with valid JWT and active subscription → 200, returns project with correct fields.

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test --test projects_test 2>&1 | tail -10`
Expected: FAIL

- [ ] **Step 3: Implement projects CRUD routes**

Implement: `list_projects`, `create_project`, `get_project`, `update_project`, `delete_project` in `routes/projects.rs`. Wire into router. All queries filter by `WHERE deleted_at IS NULL` to exclude soft-deleted projects. `delete_project` sets `deleted_at = now()` instead of hard-deleting.

- [ ] **Step 4: Write tests for list, get, update, delete, multi-tenant isolation**

Add tests: list returns only user's active projects (excludes soft-deleted), get returns project with sections, update changes title, soft-delete sets deleted_at and excludes from list, accessing other user's project returns 404.

- [ ] **Step 5: Write integration test for applicant upsert**

Create `backend/tests/applicants_test.rs` with test: PUT /api/projects/:id/applicant upserts applicant details. GET returns applicant data. Second PUT updates (not duplicates).

- [ ] **Step 6: Implement applicants routes**

```rust
// backend/api/src/routes/applicants.rs
// PUT /api/projects/:id/applicant — upsert project_applicants row (INSERT ON CONFLICT DO UPDATE)
// GET /api/projects/:id/applicant — get applicant details for a project (404 if not yet filled)
```

Wire into router under project routes. Applicant data is required for export but NOT for AI generation — the wizard can proceed past Step 2 with partial data, and the export endpoint validates completeness.

- [ ] **Step 7: Run all tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: add projects CRUD and applicant details API with multi-tenant isolation"
```

---

### Task 7: Interview responses API

**Files:**

- Create: `backend/api/src/routes/interview.rs`

- [ ] **Step 1: Write integration test for save interview**

Create `backend/tests/interview_test.rs` — PUT /api/projects/:id/interview saves batch of responses.

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement interview routes**

`save_interview` (PUT, upserts batch), `get_interview` (GET, returns in step_number order).

- [ ] **Step 4: Write tests for upsert and ordering**

Test: saving again updates (not duplicates), GET returns ordered by step_number.

- [ ] **Step 5: Run all tests, commit**

```bash
git add -A
git commit -m "feat: add interview responses API with batch upsert"
```

---

### Task 8: Frontend projects list and interview wizard

**Files:**

- Create: `apps/web/app/projects/page.tsx`
- Create: `apps/web/components/projects/project-card.tsx`
- Create: `apps/web/app/projects/new/page.tsx`
- Create: `apps/web/components/interview/interview-wizard.tsx`
- Create: `apps/web/components/interview/step-basics.tsx`
- Create: `apps/web/components/interview/step-applicant.tsx`
- Create: `apps/web/components/interview/step-problem.tsx`
- Create: `apps/web/components/interview/step-description.tsx`
- Create: `apps/web/components/interview/step-novelty.tsx`
- Create: `apps/web/components/interview/step-figures.tsx`
- Create: `apps/web/components/interview/step-review.tsx`
- Create: `apps/web/hooks/use-interview-wizard.ts`

- [ ] **Step 1: Write test for project-card component**

Test: renders title, status, date.

- [ ] **Step 2: Implement project-card**

- [ ] **Step 3: Write test for projects list page (MSW integration)**

Test: fetches and renders project list, empty state shows CTA.

- [ ] **Step 4: Implement projects list page**

- [ ] **Step 5: Write test for interview wizard step navigation**

Test: `useInterviewWizard` hook — step forward/back, data persistence.

- [ ] **Step 6: Implement useInterviewWizard hook**

Manages current step, form data per step, saves to API on step change.

- [ ] **Step 7: Write tests for each step component**

Test each step renders correct fields, validates required fields, calls onNext.

- [ ] **Step 8: Implement all 7 step components**

Step 1 (basics), Step 2 (applicant details → saves to `project_applicants`), Step 3 (problem), Step 4 (description), Step 5 (novelty), Step 6 (figures → uploads via `StorageClient` + figures API), Step 7 (review + generate button).

> **Dependency note:** Step 6 (figures) requires the storage client (Task 16) and figures API (Task 18) to be complete. If building Task 8 before Task 16/18, implement the UI with file selection and preview but disable the upload button with a "Backend not yet connected" message. Wire it up after Task 18 is complete. Alternatively, build Tasks 16+18 first (recommended — they're small) so figures work end-to-end.

- [ ] **Step 9: Implement interview wizard container**

Composes steps with progress indicator, back/next navigation.

- [ ] **Step 10: Run all frontend tests**

Run: `cd apps/web && npx vitest run 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: add projects list, interview wizard with 7 steps (incl. applicant details)"
```

---

## Phase 4: AI Drafting Pipeline (Tasks 9-11)

### Task 9: AI provider trait and Anthropic implementation

**Files:**

- Create: `backend/ai/src/provider.rs`
- Create: `backend/ai/src/anthropic.rs`
- Create: `backend/ai/src/mock.rs`

- [ ] **Step 1: Write test for mock provider**

Test: mock provider returns predetermined text as a stream.

- [ ] **Step 2: Implement LlmProvider trait**

```rust
// backend/ai/src/provider.rs
use anyhow::Result;
use tokio::sync::mpsc;

pub struct Prompt {
    pub system: String,
    pub user: String,
}

pub trait LlmProvider: Send + Sync {
    fn generate_stream(
        &self,
        prompt: Prompt,
    ) -> Result<mpsc::Receiver<Result<String>>>;
}
```

- [ ] **Step 3: Implement MockProvider**

Returns configurable text chunks via channel. Used in all integration and E2E tests.

- [ ] **Step 4: Implement AnthropicProvider**

Uses `reqwest` to call Anthropic Messages API with streaming. Parses SSE events from Anthropic, forwards text deltas through channel.

- [ ] **Step 5: Write test for provider factory**

Test: `create_provider("anthropic")` returns AnthropicProvider, `create_provider("mock")` returns MockProvider.

- [ ] **Step 6: Run tests, commit**

```bash
git add -A
git commit -m "feat: add LlmProvider trait, Anthropic and Mock implementations"
```

---

### Task 10: Prompt templates and generation pipeline

**Files:**

- Create: `backend/ai/src/prompts/mod.rs`
- Create: `backend/ai/src/prompts/title.txt` (and all 8 section prompts + cross_reference.txt)
- Create: `backend/ai/src/pipeline.rs`

- [ ] **Step 1: Write test for prompt builder**

Test: given interview responses, builds correct system + user prompt for each section type.

- [ ] **Step 2: Create prompt templates**

9 template files (8 sections + cross-reference). Each contains the system prompt with IPO formatting rules. Use `include_str!` to embed at compile time.

> **Note:** The cross-reference template (`cross_reference.txt`) is NOT a section type. It is used by a separate `run_cross_reference_check()` function in the pipeline (not via `build_prompt` with `section_type`). Its output is NOT auto-applied — it returns a list of **suggestions** (section, issue, proposed fix) that are emitted as `cross_reference_suggestion` SSE events and displayed as a review checklist in the editor UI. The user must accept/reject each suggestion.

- [ ] **Step 3: Implement prompt builder**

`build_prompt(section_type, interview_responses, previously_generated_sections, figure_descriptions) -> Prompt`

- [ ] **Step 4: Write test for pipeline orchestration**

Test with MockProvider: pipeline generates all 8 sections sequentially, returns SSE events in correct order (section_start → content_delta → section_complete for each, then generation_complete).

- [ ] **Step 5: Implement pipeline**

`GenerationPipeline::run(project_id, interview_responses, figures, provider) -> mpsc::Receiver<SseEvent>`. Generates sections sequentially, saves each to DB as it completes. Handles partial failure (keeps completed sections, returns error event).

- [ ] **Step 6: Write test for partial failure recovery**

Test: provider fails at section 4. First 3 sections are saved. Error event emitted. Project status reset.

- [ ] **Step 7: Run tests, commit**

```bash
git add -A
git commit -m "feat: add prompt templates and AI generation pipeline with error recovery"
```

---

### Task 11: Generation API endpoint and SSE streaming

**Files:**

- Create: `backend/api/src/routes/generate.rs`
- Create: `backend/api/src/routes/sections.rs`

- [ ] **Step 1: Write integration test for generate endpoint**

Test: POST /api/projects/:id/generate returns SSE stream with correct event types.

- [ ] **Step 2: Implement generate endpoint**

Uses Axum's `Sse` response type. Calls `GenerationPipeline::run()`, forwards events to client.

- [ ] **Step 3: Write integration test for section update with versioning**

Test: PUT /api/projects/:id/sections/title saves previous content to `section_versions` (version_number auto-increments), then updates content, sets ai_generated=false, increments edit_count.

- [ ] **Step 4: Implement sections routes (with version history)**

- `update_section` (PUT) — saves current content to `section_versions` before overwriting, increments version_number
- `regenerate_section` (POST) — saves current content to `section_versions` (source='ai_regenerated'), then streams new content
- `list_versions` (GET `/versions`) — returns version history sorted by version_number desc
- `restore_version` (POST `/versions/:ver/restore`) — saves current content as a version, then restores the requested version's content

- [ ] **Step 5: Write integration tests for version operations**

Test: list_versions returns correct history after multiple edits. restore_version swaps content correctly and creates a new version entry for the displaced content.

- [ ] **Step 6: Run all backend tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add AI generation endpoint with SSE streaming, section edit/regenerate, and version history"
```

---

## Phase 5: Frontend Editor (Tasks 12-13)

### Task 12: SSE stream hook and generation UI

**Files:**

- Create: `apps/web/hooks/use-sse-stream.ts`
- Create: `apps/web/components/editor/generation-stream.tsx`

- [ ] **Step 1: Write test for useSSEStream hook**

Test: parses section_start, content_delta, section_complete, error events correctly.

- [ ] **Step 2: Implement useSSEStream hook**

Connects to SSE endpoint, parses events, maintains state per section (accumulating content).

- [ ] **Step 3: Implement generation-stream component**

Shows progress: which section is generating, completed sections render immediately. Error state with retry.

- [ ] **Step 4: Run tests, commit**

```bash
git add -A
git commit -m "feat: add SSE stream hook and generation progress UI"
```

---

### Task 13: Section-based editor page

**Files:**

- Create: `apps/web/components/editor/section-card.tsx`
- Create: `apps/web/components/editor/section-list.tsx`
- Create: `apps/web/components/editor/version-history.tsx`
- Create: `apps/web/components/editor/cross-reference-checklist.tsx`
- Create: `apps/web/hooks/use-auto-save.ts`
- Create: `apps/web/app/projects/[id]/page.tsx`

- [ ] **Step 1: Write tests for section-card**

Test: renders content, toggles edit mode, regenerate confirmation, collapsed state, status badge, history button opens version panel.

- [ ] **Step 2: Implement section-card**

Card with view/edit/regenerate/history modes. Auto-save on edit via `useAutoSave`. History button opens version-history panel.

- [ ] **Step 3: Write test for version-history component**

Test: renders list of versions with timestamps and source labels, preview on click, "Restore" button calls api.restoreVersion.

- [ ] **Step 4: Implement version-history component**

Sheet/panel that shows version list from `api.listVersions()`. Each version shows: version number, timestamp, source (manual/ai_generated/ai_regenerated). Click to preview content. "Restore" button with confirmation.

- [ ] **Step 5: Write test for cross-reference-checklist component**

Test: renders list of suggestions, accept applies edit, dismiss removes suggestion.

- [ ] **Step 6: Implement cross-reference-checklist component**

Displayed after generation completes (when `cross_reference_suggestion` events were received). Each suggestion shows: section, issue, proposed fix. "Accept" applies the fix via `api.updateSection()`. "Dismiss" removes from list. Count badge shows remaining suggestions.

- [ ] **Step 7: Write test for useAutoSave**

Test: debounces calls, triggers on blur.

- [ ] **Step 8: Implement useAutoSave hook**

- [ ] **Step 9: Implement section-list**

Renders all sections in IPO Form 2 order.

- [ ] **Step 10: Implement editor page**

`/projects/[id]/page.tsx` — fetches project + sections, renders section-list. If project status is 'generating', shows generation-stream instead. After generation, shows cross-reference checklist if suggestions exist.

- [ ] **Step 11: Run all frontend tests**

Run: `cd apps/web && npx vitest run 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "feat: add section-based editor with auto-save, version history, and cross-reference checklist"
```

---

## Phase 6: Export Pipeline (Tasks 14-15)

### Task 14: PDF and DOCX generation in Rust

**Files:**

- Create: `backend/export/src/pdf.rs`
- Create: `backend/export/src/docx.rs`
- Create: `backend/export/src/templates/ipo_form2.typ`

- [ ] **Step 1: Write test for typst template generation**

Test: given patent sections, generates valid `.typ` file content with correct IPO formatting.

- [ ] **Step 2: Create IPO Form 2 typst template**

A4, numbered paragraphs, claims on separate page, abstract word count.

- [ ] **Step 3: Implement pdf.rs**

Generates `.typ` file from sections, calls `typst compile` subprocess, returns PDF bytes.

> **Template strategy:** The typst template (`ipo_form2.typ`) is embedded at compile time via `include_str!()`. At runtime, `pdf.rs` writes a temporary `.typ` file (populated with section content), calls `typst compile temp.typ output.pdf`, reads the PDF bytes, and cleans up. No runtime file path configuration needed. The Dockerfile `COPY` of templates is removed — they're baked into the binary.

- [ ] **Step 4: Add docx-rs dependency and write test**

Test: generates DOCX with correct headings and section ordering.

- [ ] **Step 5: Implement docx.rs**

Generates DOCX with styled headings, claim formatting, IPO metadata page.

- [ ] **Step 6: Run tests, commit**

```bash
git add -A
git commit -m "feat: add PDF (typst) and DOCX (docx-rs) export generation"
```

---

### Task 15: Export API endpoint and frontend page

**Files:**

- Create: `backend/api/src/routes/export.rs`
- Create: `apps/web/app/projects/[id]/export/page.tsx`

- [ ] **Step 1: Write integration test for export endpoint**

Test: POST /api/projects/:id/export with format=pdf creates file and returns export record.

- [ ] **Step 2: Implement export routes**

`create_export` (POST — generates file, uploads via `StorageClient`, saves to DB), `list_exports` (GET), `get_download_url` (GET — returns download URL from storage backend).

- [ ] **Step 3: Write test for export with missing sections**

Test: export with incomplete sections returns 400 with list of missing sections.

- [ ] **Step 4: Implement frontend export page**

Format selection buttons (PDF/DOCX), progress indicator, previous exports list with download links.

- [ ] **Step 5: Run all tests, commit**

```bash
git add -A
git commit -m "feat: add export API and frontend export page"
```

---

## Phase 7: Billing & Storage (Tasks 16-18)

### Task 16: Storage client (local + R2)

**Files:**

- Create: `backend/storage/src/client.rs`
- Create: `backend/storage/src/local.rs`
- Create: `backend/storage/src/r2.rs`

- [ ] **Step 1: Implement StorageClient trait**

```rust
// backend/storage/src/client.rs
// trait StorageClient: Send + Sync {
//     async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> Result<String>;
//     async fn download_url(&self, key: &str, expiry_secs: u64) -> Result<String>;
//     async fn delete(&self, key: &str) -> Result<()>;
// }
```

- [ ] **Step 2: Implement LocalStorage (for development)**

```rust
// backend/storage/src/local.rs
// - upload: writes to {base_path}/{key}, creates parent dirs
// - download_url: returns http://localhost:{port}/files/{key}
// - delete: removes file from disk
// Add a static file serving route in Axum: GET /files/*path → serves from STORAGE_LOCAL_PATH
```

- [ ] **Step 3: Write tests for LocalStorage**

Test: upload creates file on disk, download_url returns valid path, delete removes file.

- [ ] **Step 4: Implement R2Storage (for production)**

```rust
// backend/storage/src/r2.rs
// Uses aws-sdk-s3 with custom endpoint: https://{account_id}.r2.cloudflarestorage.com
// - upload: PutObject → returns key
// - download_url: presigned GetObject URL (1hr expiry)
// - delete: DeleteObject
```

- [ ] **Step 5: Implement factory function**

```rust
// create_storage_client(config) → Box<dyn StorageClient>
//   "local" → LocalStorage::new(config.storage_local_path)
//   "r2" → R2Storage::new(config.r2_*) — validates all R2 fields are present
```

- [ ] **Step 6: Wire StorageClient into Axum app state**

Add `StorageClient` to the Axum app state (`Arc<dyn StorageClient>`). Call `create_storage_client(&config)` in `main.rs` at startup. For local development, add a static file serving route: `GET /files/*path` that serves files from `STORAGE_LOCAL_PATH` (only enabled when `STORAGE_BACKEND=local`). This allows `LocalStorage.download_url()` to return working URLs.

- [ ] **Step 7: Run tests, commit**

```bash
git add -A
git commit -m "feat: add storage client with local filesystem (dev) and Cloudflare R2 (prod) backends"
```

---

### Task 17: Razorpay subscription setup

**Files:**

- Create: `backend/api/src/routes/subscriptions.rs`
- Create: `backend/api/src/routes/webhooks.rs`
- Create: `apps/web/lib/razorpay.ts`
- Create: `apps/web/app/subscribe/page.tsx`
- Create: `apps/web/app/account/page.tsx`

- [ ] **Step 1: Implement Razorpay subscription endpoints**

```rust
// backend/api/src/routes/subscriptions.rs
// POST /api/subscriptions/create
//   1. Create Razorpay customer (if not exists) via Razorpay API
//   2. Create Razorpay Subscription with plan_id from config
//   3. Return { subscription_id } to frontend for checkout modal
//
// POST /api/subscriptions/cancel
//   1. Look up active subscription for user
//   2. Call Razorpay API to cancel subscription
//   3. Update local subscriptions table status to 'cancelled'
//
// GET /api/subscriptions/status
//   1. Return subscription details (status, plan, period dates) from local DB
```

- [ ] **Step 2: Implement Razorpay webhook handler**

```rust
// backend/api/src/routes/webhooks.rs
// POST /api/webhooks/razorpay
// 1. Verify webhook signature (HMAC-SHA256 with RAZORPAY_WEBHOOK_SECRET)
// 2. Handle events: subscription.activated, subscription.charged,
//    subscription.cancelled, subscription.halted
// 3. Upsert to subscriptions table (INSERT ON CONFLICT DO UPDATE)
// All DB writes via sqlx — single writer, no Supabase client needed
```

- [ ] **Step 3: Write integration test for webhook handler**

Test: valid webhook event upserts subscription. Same event sent twice doesn't duplicate. Invalid signature returns 401.

- [ ] **Step 4: Implement Razorpay frontend helper**

```typescript
// apps/web/lib/razorpay.ts
// Opens Razorpay Checkout modal with subscription_id
// Loads Razorpay script dynamically
// Handles success/failure callbacks
```

- [ ] **Step 5: Implement /subscribe page**

Shows plan details (INR pricing). "Subscribe" button calls `api.createSubscription()`, then opens Razorpay Checkout modal. On success, polls `GET /api/me` until `has_active_subscription` is true, then redirects to `/projects`.

- [ ] **Step 6: Implement /account page**

Shows user profile, subscription status (active/cancelled/halted), cancel subscription button (calls Razorpay API via Rust backend).

- [ ] **Step 7: Run all tests, commit**

```bash
git add -A
git commit -m "feat: add Razorpay subscription billing, webhook handler, and account page"
```

---

### Task 18: Figures upload API

**Files:**

- Create: `backend/api/src/routes/figures.rs`

- [ ] **Step 1: Write integration test for figure upload**

Test: POST multipart upload creates DB record and stores file via `StorageClient` (uses `LocalStorage` in tests).

- [ ] **Step 2: Implement figures routes**

`upload_figure` (POST multipart → `StorageClient` + DB), `list_figures` (GET), `delete_figure` (DELETE — removes from both storage and DB).

- [ ] **Step 3: Run tests, commit**

```bash
git add -A
git commit -m "feat: add figures upload API with storage abstraction"
```

---

## Phase 8: CI/CD & Deployment (Tasks 19-20)

### Task 19: GitHub Actions CI

**Files:**

- Create: `.github/workflows/rust.yml`
- Create: `.github/workflows/frontend.yml`
- Create: `.github/workflows/e2e.yml`

- [ ] **Step 1: Create Rust CI workflow**

```yaml
# .github/workflows/rust.yml
name: Rust CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: patent_draft_pro_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    env:
      DATABASE_URL: postgresql://postgres:postgres@localhost:5432/patent_draft_pro_test
      JWT_SECRET: test-secret-for-ci-only-64-chars-long-enough
      AI_PROVIDER: mock
      STORAGE_BACKEND: local
      STORAGE_LOCAL_PATH: /tmp/patent-test-storage
      RAZORPAY_KEY_ID: rzp_test_ci
      RAZORPAY_KEY_SECRET: test
      RAZORPAY_WEBHOOK_SECRET: test
      RAZORPAY_PLAN_ID: plan_test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install sqlx-cli --no-default-features --features postgres
      - run: sqlx migrate run --source ../migrations
        working-directory: backend
      - run: cargo test --workspace
        working-directory: backend
```

- [ ] **Step 2: Create frontend CI workflow**

```yaml
# .github/workflows/frontend.yml
name: Frontend CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npm ci
        working-directory: apps/web
      - run: npx prettier --check .
        working-directory: apps/web
      - run: npm run build
        working-directory: apps/web
      - run: npm run test
        working-directory: apps/web
```

- [ ] **Step 3: Create E2E CI workflow**

E2E workflow: starts docker-compose (Postgres + Rust backend with mock LLM), runs Playwright.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "ci: add GitHub Actions workflows for Rust, frontend, and E2E tests"
```

---

### Task 20: Deployment configuration

**Files:**

- Create: `Dockerfile.backend`
- Create: `fly.toml`

- [ ] **Step 1: Create Rust backend Dockerfile**

Multi-stage build: builder (cargo build --release) → runtime (debian-slim + typst binary).

```dockerfile
# Dockerfile.backend
FROM rust:1.82 AS builder
WORKDIR /app
COPY backend/ backend/
COPY migrations/ migrations/
WORKDIR /app/backend
RUN cargo build --release -p api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
# Install typst
RUN curl -sSL https://github.com/typst/typst/releases/latest/download/typst-x86_64-unknown-linux-musl.tar.xz \
    | tar -xJ --strip-components=1 -C /usr/local/bin
COPY --from=builder /app/backend/target/release/api /usr/local/bin/api
# Templates are embedded via include_str!() at compile time — no runtime files needed
CMD ["api"]
```

- [ ] **Step 2: Create fly.toml**

```toml
# fly.toml
app = "patent-draft-pro-api"
primary_region = "bom"  # Mumbai — closest to Indian users

[build]
  dockerfile = "Dockerfile.backend"

[http_service]
  internal_port = 5012
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 1  # Keep 1 machine warm — cold starts add latency for patent generation

[env]
  RUST_LOG = "info"
  PORT = "5012"
```

- [ ] **Step 3: Verify Docker build**

Run: `docker build -f Dockerfile.backend -t patent-api . 2>&1 | tail -10`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add Dockerfile and Fly.io config for backend deployment"
```

---

## Phase 9: E2E Tests (Task 21)

### Task 21: Playwright E2E test suite

**Files:**

- Create: `e2e/playwright.config.ts`
- Create: `e2e/fixtures/test-setup.ts`
- Create: `e2e/auth.spec.ts`
- Create: `e2e/interview.spec.ts`
- Create: `e2e/generation.spec.ts`
- Create: `e2e/editor.spec.ts`
- Create: `e2e/export.spec.ts`
- Create: `e2e/projects.spec.ts`
- Create: `e2e/subscription.spec.ts`
- Create: `e2e/isolation.spec.ts`

- [ ] **Step 1: Install Playwright**

```bash
npm init -y  # root package.json for E2E
npm install -D @playwright/test
npx playwright install --with-deps chromium
```

- [ ] **Step 2: Create Playwright config**

Desktop Chrome only for MVP. Base URL points to local Next.js dev server.

- [ ] **Step 3: Create test fixtures**

Auth fixture (logs in test user), seed fixture (creates test project with active subscription).

- [ ] **Step 4: Write auth E2E tests**

Login success, login failure, unauthenticated redirect, no-subscription redirect.

- [ ] **Step 5: Write interview wizard E2E tests**

Complete wizard flow, step navigation, browser refresh resume, validation errors.

- [ ] **Step 6: Write generation E2E tests**

Generate button → sections appear with streaming, all 8 sections in correct order.

- [ ] **Step 7: Write editor E2E tests**

Edit section, cancel edit, regenerate, collapsed sections, auto-save.

- [ ] **Step 8: Write export E2E tests**

Export PDF, export DOCX, previous exports listed, download works.

- [ ] **Step 9: Write projects E2E tests**

Empty state, create project, delete project, sort order.

- [ ] **Step 10: Write subscription E2E tests**

No-subscription gate shows /subscribe page, click Subscribe opens Razorpay Checkout modal, after subscription user can access /projects.

- [ ] **Step 11: Write multi-tenant isolation E2E tests**

`isolation.spec.ts` — user A cannot see user B's projects (API returns 404), user A cannot access user B's project editor via direct URL. Uses two separate test users with separate auth fixtures.

- [ ] **Step 12: Run full E2E suite**

Run: `npx playwright test 2>&1 | tail -20`
Expected: All tests pass

- [ ] **Step 13: Commit**

```bash
git add -A
git commit -m "test: add Playwright E2E test suite for all core user flows"
```

---

## Phase 10: Polish & Ship (Task 22)

### Task 22: Final integration and README

**Files:**

- Modify: `apps/web/app/layout.tsx` (add Toaster)
- Create: `README.md`
- Modify: `Makefile` (ensure all commands work)

- [ ] **Step 1: Add toast notifications**

Add `sonner` Toaster to root layout for success/error feedback across the app.

- [ ] **Step 2: Add error boundaries**

Add error.tsx and loading.tsx for each route group.

- [ ] **Step 3: Write README**

Setup instructions, environment variables, development commands, deployment guide.

- [ ] **Step 4: Run full test suite**

```bash
make test          # Rust + frontend unit/integration
make test-e2e      # Playwright E2E
```

Expected: All green.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add toast notifications, error boundaries, and README"
```

---

## Task Dependency Graph

> **Important:** The phase headers in the plan body (Phase 1-10) reflect a logical grouping, but the **execution order** below is the actual build sequence. Storage (Task 16+18) is built in Phase 3, not Phase 7 as the header suggests.

```
Phase 1: Scaffold
  Task 1 (repo scaffold) → Task 2 (migrations incl. sessions, applicants, updated_at triggers) → Task 3 (Next.js 16 scaffold)

Phase 2: Auth
  Task 4 (Rust auth: argon2+JWT+sessions, seed-user CLI, rate limiting) → Task 5 (frontend login)

Phase 3: Core CRUD + Storage (storage built early — needed for figures in wizard)
  Task 6 (projects + applicants API, soft-delete) → Task 7 (interview API)
  Task 16 (storage client: local+R2, wired into Axum) → Task 18 (figures upload)
  Task 8 (frontend wizard with 7 steps, figures wired to storage API)

Phase 4: AI Pipeline
  Task 9 (provider trait) → Task 10 (prompts + pipeline + cross-ref + abstract validation) → Task 11 (generate endpoint + section versioning)

Phase 5: Editor
  Task 12 (SSE hook) → Task 13 (editor page + version history + cross-ref checklist)

Phase 6: Export
  Task 14 (PDF/DOCX generation with applicant title page) → Task 15 (export endpoint + page)

Phase 7: Billing
  Task 17 (Razorpay billing + cancel/status + webhooks in Rust) — can run in parallel with Phases 4-6

Phase 8: CI/CD
  Task 19 (GitHub Actions) → Task 20 (Dockerfile + Fly.io, production CORS)

Phase 9: E2E
  Task 21 (Playwright) — after all features complete

Phase 10: Polish
  Task 22 (final integration) — last
```

**Parallelization opportunities:**

- Task 16+18 (storage client + figures) should complete BEFORE Task 8 (wizard) so Step 6 works end-to-end
- Task 17 (Razorpay) can run in parallel with Tasks 9-15 (AI pipeline + editor + export)
- Tasks 19-20 (CI/CD) can start after Phase 3 is complete
- Frontend tasks (5, 8, 12-13, 15, 17) can partially overlap with backend tasks if APIs are defined first
- Consider writing 1-2 smoke E2E tests (login → projects list) after Phase 3 to catch integration issues early; the full E2E suite (Task 21) runs after all features are complete

**Critical path:** Tasks 1→2→3→4→5→6→7→16→18→8→9→10→11→12→13→14→15→21→22

**Estimated effort:** ~22 tasks across 10 phases. With focused execution: ~7-9 weeks for a single developer.
