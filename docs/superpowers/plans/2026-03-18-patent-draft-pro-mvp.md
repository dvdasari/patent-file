# Patent Draft Pro — MVP Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an AI-powered patent drafting SaaS for Indian patent agents — guided interview → AI-generated draft → section editor → PDF/DOCX export.

**Architecture:** Rust (Axum) backend with provider-agnostic AI pipeline and SSE streaming, Next.js 15 frontend with section-based card editor, Supabase for Postgres/Auth/Storage, Stripe for billing. Monorepo with Cargo workspace (backend) + npm workspace (frontend).

**Tech Stack:** Rust (Axum, sqlx, tokio, reqwest), Next.js 15 (App Router), React, shadcn/ui, Tailwind CSS, Supabase, Stripe, Playwright, Vitest, typst CLI, docx-rs

**Spec:** `docs/superpowers/specs/2026-03-18-patent-draft-pro-mvp-design.md`

**New Repo:** This plan is for a new repository `patent-draft-pro` (NOT the dasari-ai repo).

---

## File Structure

```
patent-draft-pro/
├── apps/
│   └── web/                              ← Next.js 15 (App Router)
│       ├── app/
│       │   ├── layout.tsx                ← Root layout (Geist fonts, providers)
│       │   ├── login/page.tsx            ← Login form
│       │   ├── subscribe/page.tsx        ← Subscription gate
│       │   ├── projects/
│       │   │   ├── page.tsx              ← Project list dashboard
│       │   │   ├── new/page.tsx          ← Interview wizard
│       │   │   └── [id]/
│       │   │       ├── page.tsx          ← Section-based editor
│       │   │       └── export/page.tsx   ← Export page
│       │   ├── account/page.tsx          ← Profile + billing
│       │   └── api/
│       │       └── webhooks/
│       │           └── stripe/route.ts   ← Stripe webhook handler
│       ├── components/
│       │   ├── ui/                       ← shadcn/ui primitives
│       │   ├── interview/
│       │   │   ├── interview-wizard.tsx  ← Wizard container + step routing
│       │   │   ├── step-basics.tsx       ← Step 1: title, type, field
│       │   │   ├── step-problem.tsx      ← Step 2: problem, prior art
│       │   │   ├── step-description.tsx  ← Step 3: invention description
│       │   │   ├── step-novelty.tsx      ← Step 4: novelty, advantages
│       │   │   ├── step-figures.tsx      ← Step 5: figure uploads
│       │   │   └── step-review.tsx       ← Step 6: review + generate
│       │   ├── editor/
│       │   │   ├── section-card.tsx      ← Single section card (view/edit/regenerate)
│       │   │   ├── section-list.tsx      ← Ordered list of all sections
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
│       │   └── use-auto-save.ts          ← Debounced auto-save
│       ├── lib/
│       │   ├── api-client.ts             ← Typed HTTP client for Rust backend
│       │   ├── supabase-client.ts        ← Supabase browser client
│       │   ├── supabase-server.ts        ← Supabase server client
│       │   └── stripe.ts                 ← Stripe checkout helpers
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
│   │       │   ├── me.rs                 ← GET/POST /api/me
│   │       │   ├── projects.rs           ← CRUD /api/projects
│   │       │   ├── interview.rs          ← GET/PUT /api/projects/:id/interview
│   │       │   ├── figures.rs            ← POST/GET/DELETE /api/projects/:id/figures
│   │       │   ├── generate.rs           ← POST /api/projects/:id/generate (SSE)
│   │       │   ├── sections.rs           ← PUT/POST /api/projects/:id/sections
│   │       │   └── export.rs             ← POST/GET /api/projects/:id/export
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs               ← JWT validation
│   │       │   └── subscription.rs       ← Subscription check
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
│   ├── 002_create_projects.sql
│   ├── 003_create_interview_responses.sql
│   ├── 004_create_patent_sections.sql
│   ├── 005_create_figures.sql
│   ├── 006_create_exports.sql
│   └── 007_create_subscriptions.sql
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
│   └── export_test.rs
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
SUPABASE_JWT_SECRET=your-supabase-jwt-secret
SUPABASE_URL=https://xxx.supabase.co
SUPABASE_SERVICE_KEY=your-supabase-service-key
ANTHROPIC_API_KEY=sk-ant-xxx
OPENAI_API_KEY=sk-xxx          # optional, for future use
AI_PROVIDER=anthropic           # "anthropic" (default) or "mock" for testing
RUST_LOG=info
PORT=3001

# Frontend
NEXT_PUBLIC_SUPABASE_URL=https://xxx.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=your-supabase-anon-key
NEXT_PUBLIC_API_URL=http://localhost:3001
STRIPE_SECRET_KEY=sk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx
STRIPE_PRICE_ID=price_xxx
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
members = ["api", "ai", "export", "shared"]
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
    pub supabase_jwt_secret: String,
    pub supabase_url: String,
    pub supabase_service_key: String,
    pub anthropic_api_key: String,
    pub port: u16,
    pub ai_provider: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            supabase_jwt_secret: std::env::var("SUPABASE_JWT_SECRET")
                .context("SUPABASE_JWT_SECRET must be set")?,
            supabase_url: std::env::var("SUPABASE_URL")
                .context("SUPABASE_URL must be set")?,
            supabase_service_key: std::env::var("SUPABASE_SERVICE_KEY")
                .context("SUPABASE_SERVICE_KEY must be set")?,
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY")
                .context("ANTHROPIC_API_KEY must be set")?,
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .context("PORT must be a valid u16")?,
            ai_provider: std::env::var("AI_PROVIDER")
                .unwrap_or_else(|_| "anthropic".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_port() {
        // Clear PORT to test default
        std::env::remove_var("PORT");
        std::env::set_var("DATABASE_URL", "postgresql://test");
        std::env::set_var("SUPABASE_JWT_SECRET", "test");
        std::env::set_var("SUPABASE_URL", "https://test.supabase.co");
        std::env::set_var("SUPABASE_SERVICE_KEY", "test");
        std::env::set_var("ANTHROPIC_API_KEY", "test");
        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.port, 3001);
    }

    #[test]
    fn test_default_ai_provider() {
        std::env::remove_var("AI_PROVIDER");
        std::env::set_var("DATABASE_URL", "postgresql://test");
        std::env::set_var("SUPABASE_JWT_SECRET", "test");
        std::env::set_var("SUPABASE_URL", "https://test.supabase.co");
        std::env::set_var("SUPABASE_SERVICE_KEY", "test");
        std::env::set_var("ANTHROPIC_API_KEY", "test");
        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.ai_provider, "anthropic");
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

    let app = Router::new()
        .route("/api/health", get(health))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

- [ ] **Step 8: Create stub ai and export crates**

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

```makefile
.PHONY: dev dev-backend dev-frontend db-up db-down migrate test test-backend test-frontend test-e2e

db-up:
	docker-compose up -d

db-down:
	docker-compose down

migrate:
	cd backend && sqlx migrate run --source ../migrations

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
```

- [ ] **Step 10: Verify Rust workspace compiles**

Run: `cd backend && cargo build 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 11: Verify health endpoint**

Run: `cd backend && cargo run -p api &` then `curl http://localhost:3001/api/health`
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
- Create: `migrations/002_create_projects.sql`
- Create: `migrations/003_create_interview_responses.sql`
- Create: `migrations/004_create_patent_sections.sql`
- Create: `migrations/005_create_figures.sql`
- Create: `migrations/006_create_exports.sql`
- Create: `migrations/007_create_subscriptions.sql`
- Create: `backend/shared/src/models.rs`
- Create: `backend/shared/src/db.rs`

- [ ] **Step 1: Create migration files**

```sql
-- migrations/001_create_users.sql
CREATE TABLE users (
    id              UUID PRIMARY KEY,
    email           TEXT UNIQUE NOT NULL,
    full_name       TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
-- migrations/002_create_projects.sql
CREATE TABLE projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'draft',
    jurisdiction    TEXT NOT NULL DEFAULT 'IPO',
    patent_type     TEXT NOT NULL DEFAULT 'complete',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_projects_user_id ON projects(user_id);
```

```sql
-- migrations/003_create_interview_responses.sql
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
-- migrations/004_create_patent_sections.sql
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
-- migrations/005_create_figures.sql
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
-- migrations/006_create_exports.sql
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
-- migrations/007_create_subscriptions.sql
CREATE TABLE subscriptions (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                  UUID NOT NULL REFERENCES users(id),
    stripe_customer_id       TEXT NOT NULL,
    stripe_subscription_id   TEXT NOT NULL UNIQUE,
    plan_id                  TEXT NOT NULL,
    status                   TEXT NOT NULL DEFAULT 'active',
    current_period_start     TIMESTAMPTZ NOT NULL,
    current_period_end       TIMESTAMPTZ NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_stripe_subscription_id ON subscriptions(stripe_subscription_id);
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
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub status: String,
    pub jurisdiction: String,
    pub patent_type: String,
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
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_customer_id: String,
    pub stripe_subscription_id: String,
    pub plan_id: String,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
Expected: All 7 migrations applied successfully

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
- Create: `apps/web/lib/supabase-client.ts`
- Create: `apps/web/lib/supabase-server.ts`
- Create: `apps/web/lib/api-client.ts`

- [ ] **Step 1: Create Next.js app**

```bash
cd apps
npx create-next-app@latest web --typescript --tailwind --eslint --app --src-dir=false --import-alias="@/*"
```

- [ ] **Step 2: Install dependencies**

```bash
cd apps/web
npm install @supabase/supabase-js @supabase/ssr stripe
npm install -D vitest @testing-library/react @testing-library/jest-dom jsdom msw @vitejs/plugin-react
```

- [ ] **Step 3: Initialize shadcn/ui**

```bash
cd apps/web
npx shadcn@latest init
# Select: New York style, Zinc base color, CSS variables
npx shadcn@latest add button input textarea card label select dialog toast
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

- [ ] **Step 5: Create Supabase client helpers**

```typescript
// apps/web/lib/supabase-client.ts
import { createBrowserClient } from "@supabase/ssr";

export function createClient() {
  return createBrowserClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
  );
}
```

```typescript
// apps/web/lib/supabase-server.ts
import { createServerClient } from "@supabase/ssr";
import { cookies } from "next/headers";

export async function createServerSupabaseClient() {
  const cookieStore = await cookies();
  return createServerClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    {
      cookies: {
        getAll() {
          return cookieStore.getAll();
        },
        setAll(cookiesToSet) {
          cookiesToSet.forEach(({ name, value, options }) => {
            cookieStore.set(name, value, options);
          });
        },
      },
    },
  );
}
```

- [ ] **Step 6: Set up MSW test infrastructure**

Create `apps/web/test/mocks/handlers.ts` (empty handler array to start), `apps/web/test/mocks/server.ts` (MSW `setupServer` with handlers), and `apps/web/test/setup.ts` (starts MSW server before all tests, resets handlers after each, closes after all). Update `vitest.config.ts` to include `setupFiles: ['./test/setup.ts']` and set `environment: 'jsdom'`.

- [ ] **Step 7: Create typed API client**

```typescript
// apps/web/lib/api-client.ts
import { createClient } from "./supabase-client";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

async function getAuthHeaders(): Promise<Record<string, string>> {
  const supabase = createClient();
  const {
    data: { session },
  } = await supabase.auth.getSession();
  if (!session?.access_token) {
    throw new Error("Not authenticated");
  }
  return {
    Authorization: `Bearer ${session.access_token}`,
    "Content-Type": "application/json",
  };
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers = await getAuthHeaders();
  const res = await fetch(`${API_URL}${path}`, {
    ...options,
    headers: { ...headers, ...options.headers },
  });

  if (res.status === 401) {
    throw new Error("Unauthorized");
  }
  if (res.status === 403) {
    throw new Error("Subscription required");
  }
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.error || `Request failed: ${res.status}`);
  }

  return res.json();
}

export const api = {
  // User — upsertMe sends email + full_name from Supabase session
  upsertMe: (data: { email: string; full_name: string }) =>
    request("/api/me", { method: "POST", body: JSON.stringify(data) }),
  getMe: () => request("/api/me"),

  // Projects
  listProjects: () => request("/api/projects"),
  createProject: (data: { title: string; patent_type: string; jurisdiction?: string }) =>
    request("/api/projects", { method: "POST", body: JSON.stringify(data) }),
  getProject: (id: string) => request(`/api/projects/${id}`),
  updateProject: (id: string, data: Record<string, string>) =>
    request(`/api/projects/${id}`, { method: "PATCH", body: JSON.stringify(data) }),
  deleteProject: (id: string) => request(`/api/projects/${id}`, { method: "DELETE" }),

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
git commit -m "feat: scaffold Next.js frontend with Supabase, shadcn/ui, and API client"
```

---

## Phase 2: Auth & Middleware (Tasks 4-5)

### Task 4: Rust JWT auth middleware

**Files:**

- Create: `backend/api/src/middleware/mod.rs`
- Create: `backend/api/src/middleware/auth.rs`
- Create: `backend/api/src/middleware/subscription.rs`
- Create: `backend/api/src/error.rs`
- Create: `backend/api/src/routes/mod.rs`
- Create: `backend/api/src/routes/health.rs`
- Create: `backend/api/src/routes/me.rs`

- [ ] **Step 1: Add jwt dependencies to api Cargo.toml**

Add to `backend/api/Cargo.toml`:

```toml
jsonwebtoken = "9"
chrono.workspace = true
```

- [ ] **Step 2: Write failing test for JWT validation**

Create `backend/api/src/middleware/auth.rs` with test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_id_from_valid_claims() {
        let user_id = uuid::Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: chrono::Utc::now().timestamp() as usize + 3600,
            iss: None,
        };
        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[test]
    fn test_reject_expired_token() {
        let secret = "test-secret";
        let user_id = uuid::Uuid::new_v4();
        let token = create_test_token(secret, &user_id.to_string(), -3600); // expired 1hr ago
        let result = validate_token(&token, secret);
        assert!(result.is_err());
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd backend && cargo test -p api 2>&1 | tail -10`
Expected: FAIL — `Claims` not defined

- [ ] **Step 4: Implement JWT middleware**

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
    pub iss: Option<String>,
}

impl Claims {
    pub fn user_id(&self) -> anyhow::Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|e| anyhow::anyhow!("Invalid user ID in token: {}", e))
    }
}

pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["supabase"]);
    validation.validate_iss = false; // Supabase JWT may not always have iss

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

pub async fn auth_middleware(
    State(jwt_secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = validate_token(token, &jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = claims.user_id().map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthUser { user_id });
    Ok(next.run(request).await)
}

#[cfg(test)]
pub fn create_test_token(secret: &str, sub: &str, expires_in_secs: i64) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: sub.to_string(),
        exp: (chrono::Utc::now().timestamp() + expires_in_secs) as usize,
        iss: Some("supabase".to_string()),
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
            iss: None,
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

- [ ] **Step 7: Implement /api/me endpoint**

```rust
// backend/api/src/routes/me.rs
use axum::{extract::State, http::StatusCode, Json, Extension};
use serde::{Deserialize, Serialize};
use shared::models::User;
use sqlx::PgPool;
use crate::middleware::auth::AuthUser;

#[derive(Serialize)]
pub struct MeResponse {
    pub user: User,
    pub has_active_subscription: bool,
}

pub async fn get_me(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
) -> Result<Json<MeResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
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

    Ok(Json(MeResponse { user, has_active_subscription }))
}

#[derive(Deserialize)]
pub struct UpsertMeRequest {
    pub email: String,
    pub full_name: String,
}

pub async fn upsert_me(
    Extension(auth): Extension<AuthUser>,
    State(pool): State<PgPool>,
    Json(body): Json<UpsertMeRequest>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, full_name) VALUES ($1, $2, $3)
         ON CONFLICT (id) DO UPDATE SET email = $2, full_name = $3, updated_at = now()
         RETURNING *"
    )
    .bind(auth.user_id)
    .bind(&body.email)
    .bind(&body.full_name)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}
```

- [ ] **Step 8: Wire routes and middleware into main.rs**

Update `backend/api/src/main.rs` to compose the router with auth middleware on protected routes and no-auth on /api/health and /api/me.

- [ ] **Step 9: Run all tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: add JWT auth middleware, subscription guard, and /api/me endpoint"
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

Build login page with Supabase email/password auth, error display, redirect to /projects on success (calls `api.upsertMe()` after login).

- [ ] **Step 4: Implement auth guard**

Server component that checks Supabase session, redirects to /login if no session.

- [ ] **Step 5: Implement subscription guard**

Server component that calls `GET /api/me`, redirects to /subscribe if `has_active_subscription` is false.

> **Important:** This runs as a Next.js server component, so `supabase.auth.getSession()` (browser API) won't work. Instead, use `supabase-server.ts` to read the session from cookies, extract the access token, and pass it directly as `Authorization: Bearer <token>` when calling the Rust backend. Create a separate `lib/api-server.ts` helper for server-side Rust API calls that reads the token from the server Supabase client.

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

### Task 6: Projects CRUD API

**Files:**

- Create: `backend/api/src/routes/projects.rs`

- [ ] **Step 1: Write integration test for create project**

Create `backend/tests/projects_test.rs` with test: POST /api/projects with valid JWT and active subscription → 200, returns project with correct fields.

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test --test projects_test 2>&1 | tail -10`
Expected: FAIL

- [ ] **Step 3: Implement projects CRUD routes**

Implement: `list_projects`, `create_project`, `get_project`, `update_project`, `delete_project` in `routes/projects.rs`. Wire into router.

- [ ] **Step 4: Write tests for list, get, update, delete, multi-tenant isolation**

Add tests: list returns only user's projects, get returns project with sections, update changes title, delete cascades, accessing other user's project returns 404.

- [ ] **Step 5: Run all tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add projects CRUD API with multi-tenant isolation"
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

- [ ] **Step 8: Implement all 6 step components**

Step 1 (basics), Step 2 (problem), Step 3 (description), Step 4 (novelty), Step 5 (figures), Step 6 (review + generate button).

> **Note for Step 5 (figures):** The figures upload backend (Task 17) doesn't exist yet at this point. Implement the UI component with file selection and preview, but stub the actual upload call with a TODO comment. The upload will be wired in Task 17. The wizard should still proceed past Step 5 even without a working upload.

- [ ] **Step 9: Implement interview wizard container**

Composes steps with progress indicator, back/next navigation.

- [ ] **Step 10: Run all frontend tests**

Run: `cd apps/web && npx vitest run 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: add projects list, interview wizard with 6 steps"
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

> **Note:** The cross-reference template (`cross_reference.txt`) is NOT a section type. It is used by a separate `run_cross_reference_pass()` function in the pipeline (not via `build_prompt` with `section_type`). Its output is not stored as a `patent_sections` row — instead, it returns a list of suggested corrections that the pipeline applies as updates to existing sections.

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

- [ ] **Step 3: Write integration test for section update**

Test: PUT /api/projects/:id/sections/title updates content, sets ai_generated=false, increments edit_count.

- [ ] **Step 4: Implement sections routes**

`update_section` (PUT), `regenerate_section` (POST, returns SSE for single section).

- [ ] **Step 5: Run all backend tests**

Run: `cd backend && cargo test --workspace 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add AI generation endpoint with SSE streaming and section edit/regenerate"
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
- Create: `apps/web/hooks/use-auto-save.ts`
- Create: `apps/web/app/projects/[id]/page.tsx`

- [ ] **Step 1: Write tests for section-card**

Test: renders content, toggles edit mode, regenerate confirmation, collapsed state, status badge.

- [ ] **Step 2: Implement section-card**

Card with view/edit/regenerate modes. Auto-save on edit via `useAutoSave`.

- [ ] **Step 3: Write test for useAutoSave**

Test: debounces calls, triggers on blur.

- [ ] **Step 4: Implement useAutoSave hook**

- [ ] **Step 5: Implement section-list**

Renders all sections in IPO Form 2 order.

- [ ] **Step 6: Implement editor page**

`/projects/[id]/page.tsx` — fetches project + sections, renders section-list. If project status is 'generating', shows generation-stream instead.

- [ ] **Step 7: Run all frontend tests**

Run: `cd apps/web && npx vitest run 2>&1 | tail -10`
Expected: All pass

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: add section-based editor with auto-save and regenerate"
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

`create_export` (POST — generates file, uploads to Supabase Storage, saves to DB), `list_exports` (GET), `get_download_url` (GET — returns signed URL).

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

## Phase 7: Billing (Tasks 16-17)

### Task 16: Stripe subscription setup

**Files:**

- Create: `apps/web/lib/stripe.ts`
- Create: `apps/web/app/subscribe/page.tsx`
- Create: `apps/web/app/api/webhooks/stripe/route.ts`
- Create: `apps/web/app/account/page.tsx`

- [ ] **Step 1: Install Stripe dependencies**

```bash
cd apps/web && npm install stripe @stripe/stripe-js
```

- [ ] **Step 2: Implement Stripe checkout helper**

`lib/stripe.ts` — creates Checkout Session (server-side), redirects to Stripe.

- [ ] **Step 3: Implement /subscribe page**

Shows plan details, "Subscribe" button creates Stripe Checkout session and redirects. Success URL: `/projects`, cancel URL: `/subscribe`.

- [ ] **Step 4: Implement Stripe webhook handler**

`app/api/webhooks/stripe/route.ts` — verifies Stripe signature, handles `checkout.session.completed`, `customer.subscription.updated`, `customer.subscription.deleted`. Uses upsert (`INSERT ... ON CONFLICT DO UPDATE`) for idempotency. Writes directly to Supabase Postgres via Supabase client.

- [ ] **Step 5: Write test for webhook idempotency**

Test: sending same webhook event twice doesn't duplicate subscription.

- [ ] **Step 6: Implement /account page**

Shows user profile, subscription status (active/canceled/past_due), "Manage Billing" button links to Stripe Customer Portal.

- [ ] **Step 7: Run all tests, commit**

```bash
git add -A
git commit -m "feat: add Stripe subscription billing, webhook handler, and account page"
```

---

### Task 17: Figures upload API

**Files:**

- Create: `backend/api/src/routes/figures.rs`

- [ ] **Step 1: Write integration test for figure upload**

Test: POST multipart upload creates DB record and stores file.

- [ ] **Step 2: Implement figures routes**

`upload_figure` (POST multipart → Supabase Storage + DB), `list_figures` (GET), `delete_figure` (DELETE).

- [ ] **Step 3: Run tests, commit**

```bash
git add -A
git commit -m "feat: add figures upload API with Supabase Storage"
```

---

## Phase 8: CI/CD & Deployment (Tasks 18-19)

### Task 18: GitHub Actions CI

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
      SUPABASE_JWT_SECRET: test-secret
      SUPABASE_URL: https://test.supabase.co
      SUPABASE_SERVICE_KEY: test-key
      ANTHROPIC_API_KEY: test-key
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install sqlx-cli --no-default-features --features postgres
      - run: sqlx migrate run --source migrations
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

### Task 19: Deployment configuration

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
  internal_port = 3001
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 1  # Keep 1 machine warm — cold starts add latency for patent generation

[env]
  RUST_LOG = "info"
  PORT = "3001"
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

## Phase 9: E2E Tests (Task 20)

### Task 20: Playwright E2E test suite

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

No-subscription gate shows /subscribe page, click Subscribe redirects to Stripe Checkout (verify URL), after subscription user can access /projects.

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

## Phase 10: Polish & Ship (Task 21)

### Task 21: Final integration and README

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

```
Phase 1: Scaffold
  Task 1 (repo scaffold) → Task 2 (migrations) → Task 3 (Next.js scaffold)

Phase 2: Auth
  Task 4 (Rust auth middleware) → Task 5 (frontend login)

Phase 3: Core CRUD
  Task 6 (projects API) → Task 7 (interview API) → Task 8 (frontend wizard)

Phase 4: AI Pipeline
  Task 9 (provider trait) → Task 10 (prompts + pipeline) → Task 11 (generate endpoint)

Phase 5: Editor
  Task 12 (SSE hook) → Task 13 (editor page)

Phase 6: Export
  Task 14 (PDF/DOCX generation) → Task 15 (export endpoint + page)

Phase 7: Billing
  Task 16 (Stripe) — can be parallel with Phase 4-6
  Task 17 (figures upload) — can be parallel with Phase 4

Phase 8: CI/CD
  Task 18 (GitHub Actions) → Task 19 (Dockerfile + Fly.io)

Phase 9: E2E
  Task 20 (Playwright) — after all features complete

Phase 10: Polish
  Task 21 (final integration) — last
```

**Parallelization opportunities:**

- Tasks 16-17 (billing + figures) can run in parallel with Tasks 9-15 (AI pipeline + editor + export)
- Tasks 18-19 (CI/CD) can start after Phase 3 is complete
- Frontend tasks (5, 8, 12-13, 15-16) can partially overlap with backend tasks if APIs are defined first

**Estimated effort:** ~21 tasks across 10 phases. With focused execution: ~6-8 weeks for a single developer.
