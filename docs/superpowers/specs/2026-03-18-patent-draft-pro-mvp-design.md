# Patent Draft Pro — MVP Design Specification

**Date:** 2026-03-18
**Status:** Draft
**Author:** AI-assisted design with user approval

---

## 1. Overview

Patent Draft Pro is a SaaS application that helps patent agents and attorneys draft patent applications using AI. The MVP focuses on **AI-powered patent drafting** for the **Indian Patent Office (IPO)** format, targeting **solo practitioners**.

### Product Vision

Similar to [ipauthor.com](https://ipauthor.com) but focused on the Indian patent market — an underserved segment. Users complete a guided interview about their invention, AI generates a complete patent application (claims, specification, abstract, figures description), and users review/edit in a section-based editor before exporting as PDF or Word.

### MVP Scope

**In scope:**

- AI patent drafting (guided interview → AI-generated draft → section editor → export)
- IPO jurisdiction (provisional + complete specifications, Form 2 format)
- Solo practitioner accounts (single user, no teams)
- Email/password authentication (private — no public signup for MVP), self-managed in Rust (argon2 + JWT)
- Razorpay subscription billing (single plan, INR pricing, UPI + cards)
- PDF + DOCX export formatted to IPO standards
- Provider-agnostic AI (Claude default, swappable)

**Out of scope (post-MVP):**

- Office Action Response module
- Prior Art Search module
- Invention Disclosure Intake module
- PCT National Phase / US (USPTO) / EPO jurisdictions
- Multi-user teams and collaboration
- Admin dashboard and analytics
- Marketing/landing page
- Usage-based billing
- Direct IPO e-filing integration
- Real-time collaboration (multi-user editing)
- OAuth/social login

---

## 2. Target User

**Solo patent agents and attorneys in India** who:

- Draft 5-20+ patent applications per month
- Currently write specifications manually in Word
- Want to reduce drafting time from days to hours
- Are comfortable with web-based tools
- Need output in IPO Form 2 format

---

## 3. Architecture

### System Diagram

```
┌─────────────────────────────────────────────────────────┐
│                      FRONTEND                            │
│              Next.js 16 (App Router)                     │
│              Vercel                                      │
│                                                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Login   │  │  Interview   │  │  Draft Editor     │  │
│  │  Page    │  │  Wizard      │  │  (Section Cards)  │  │
│  └──────────┘  └──────────────┘  └──────────────────┘  │
│                        │                    │            │
│                    REST API calls + SSE streaming        │
└────────────────────────┼────────────────────┼───────────┘
                         │                    │
                    HTTPS (JSON + SSE)
                         │                    │
┌────────────────────────┼────────────────────┼───────────┐
│                      BACKEND                             │
│                 Rust (Axum)                               │
│                 Fly.io (Mumbai region)                    │
│                                                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Auth    │  │  AI Service  │  │  Export Service   │  │
│  │ (argon2  │  │  (LLM calls) │  │  (PDF/DOCX gen)  │  │
│  │  + JWT)  │  │              │  │                   │  │
│  └──────────┘  └──────────────┘  └──────────────────┘  │
│       │               │                    │            │
│  ┌────┴───────────────┴────────────────────┴─────┐     │
│  │              sqlx (single DB writer)           │     │
│  └────────────────────┬──────────────────────────┘     │
│                        │                                │
│  ┌─────────────────────┤  ┌──────────────────────────┐ │
│  │  Razorpay webhooks  │  │  StorageClient           │ │
│  │  (signature verify) │  │  (Local FS dev / R2 prod)│ │
│  └─────────────────────┘  └──────────────────────────┘ │
└────────────────────────┼────────────────────────────────┘
                         │
              ┌──────────┴──────────┐
              │  Supabase Postgres  │
              │  (managed DB only)  │
              └─────────────────────┘
```

### Key Architectural Decisions

1. **AI calls go through Rust backend** — not directly from frontend. Controls rate limiting, prompt engineering, and token usage server-side.
2. **Streaming via SSE** — AI-generated text streams from Rust → Next.js frontend so users see text appear in real-time.
3. **Postgres via Docker locally, Supabase in production** — Rust connects via sqlx. No Supabase Auth, no Supabase Storage. Auth is self-managed in Rust (argon2 password hashing + JWT issuance). File storage uses Cloudflare R2 in production, local filesystem in development.
4. **Provider-agnostic AI layer** — trait-based abstraction in Rust. Start with Claude (Anthropic), stub OpenAI. Swap or A/B test providers via config.
5. **All webhooks in Rust** — Razorpay webhooks are handled by the Rust backend directly. Single DB writer (sqlx), no split-brain between frontend and backend database access.
6. **Cloudflare R2 for file storage (production)** — S3-compatible API, zero egress fees. Stores exported PDFs/DOCX files and uploaded figure images. In local development, a `LocalStorage` backend writes files to `./storage/` on disk — no R2 credentials needed.
7. **Application-level rate limiting** — per-user limits on expensive operations (AI generation, export) enforced in Rust middleware. Vercel Firewall provides edge-level rate limiting on frontend routes.
8. **CORS locked to frontend origin** — configured via `ALLOWED_ORIGIN` environment variable (`http://localhost:3000` for local dev, production Vercel URL in prod). The Rust backend uses `CorsLayer` with `allow_credentials(true)` so httpOnly cookies are sent cross-origin. Never use `CorsLayer::permissive()` — it disables credential support.
9. **Soft-delete for projects** — projects use a `deleted_at` column instead of hard DELETE. Patent drafts represent hours of user work; accidental deletion must be recoverable. Cascade deletes only occur when a user explicitly purges from a "Trash" view (post-MVP) or after 30 days.

---

## 4. Database Schema

**Note:** All tables with `updated_at` columns use a shared Postgres trigger function (`set_updated_at()`) that automatically sets `updated_at = now()` on every UPDATE. This is created in migration 001 before any table definitions. The application layer does NOT need to set `updated_at` manually.

```sql
-- Auto-update trigger (applied to all tables with updated_at)
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Users (self-managed auth — no Supabase Auth dependency)
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT UNIQUE NOT NULL,
    password_hash   TEXT NOT NULL,  -- argon2 hash
    full_name       TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sessions (server-side refresh token tracking + revocation)
CREATE TABLE sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL,   -- SHA-256 hash of the refresh token (never store raw)
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked         BOOLEAN NOT NULL DEFAULT false,
    user_agent      TEXT,               -- browser/device info for session management
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Patent projects (one per invention — soft-delete via deleted_at)
CREATE TABLE projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'draft',
        -- 'draft' | 'interview_complete' | 'generating' | 'review' | 'exported'
    jurisdiction    TEXT NOT NULL DEFAULT 'IPO',
    patent_type     TEXT NOT NULL DEFAULT 'complete',
        -- 'provisional' | 'complete'
    deleted_at      TIMESTAMPTZ,  -- NULL = active, set = soft-deleted
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Applicant/filing metadata (required for IPO Form 2 title page)
CREATE TABLE project_applicants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    applicant_name  TEXT NOT NULL,
    applicant_address TEXT NOT NULL,
    applicant_nationality TEXT NOT NULL DEFAULT 'Indian',
    inventor_name   TEXT NOT NULL,
    inventor_address TEXT NOT NULL,
    inventor_nationality TEXT NOT NULL DEFAULT 'Indian',
    agent_name      TEXT,               -- patent agent name (optional if self-filing)
    agent_registration_no TEXT,         -- IPO agent registration number
    assignee_name   TEXT,               -- if different from applicant
    priority_date   DATE,               -- priority claim date (if any)
    priority_country TEXT,              -- priority claim country
    priority_application_no TEXT,       -- priority application number
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id)                  -- one applicant record per project
);

-- Interview responses (structured invention disclosure)
CREATE TABLE interview_responses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    step_number     INT NOT NULL,
    question_key    TEXT NOT NULL,
    question_text   TEXT NOT NULL,
    response_text   TEXT,  -- nullable for optional fields (e.g., alternative embodiments)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, question_key)
);

-- Generated patent sections (AI output, one row per section)
CREATE TABLE patent_sections (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    section_type    TEXT NOT NULL,
        -- 'title' | 'abstract' | 'field_of_invention' | 'background'
        -- | 'summary' | 'detailed_description' | 'claims' | 'drawings_description'
    content         TEXT NOT NULL,
    ai_generated    BOOLEAN NOT NULL DEFAULT true,
    edit_count      INT NOT NULL DEFAULT 0,  -- mutation counter, incremented on each edit/regeneration
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(project_id, section_type)
);

-- Section versions (history for undo/restore)
CREATE TABLE section_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_id      UUID NOT NULL REFERENCES patent_sections(id) ON DELETE CASCADE,
    content         TEXT NOT NULL,
    version_number  INT NOT NULL,
    source          TEXT NOT NULL DEFAULT 'manual',  -- 'manual' | 'ai_generated' | 'ai_regenerated'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(section_id, version_number)
);

-- Figures (uploaded invention sketches/diagrams)
CREATE TABLE figures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    sort_order      INT NOT NULL DEFAULT 0,
    description     TEXT NOT NULL,  -- user-provided figure description
    storage_path    TEXT NOT NULL,  -- storage key (local path or R2 object key)
    file_name       TEXT NOT NULL,
    content_type    TEXT NOT NULL,  -- e.g., 'image/png'
    file_size_bytes BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Exports (generated files)
CREATE TABLE exports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    format          TEXT NOT NULL,  -- 'pdf' | 'docx'
    storage_path    TEXT NOT NULL,  -- storage key (local path or R2 object key)
    file_size_bytes BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Subscriptions (Razorpay state cached locally)
CREATE TABLE subscriptions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID NOT NULL REFERENCES users(id),
    razorpay_customer_id        TEXT NOT NULL,
    razorpay_subscription_id    TEXT NOT NULL UNIQUE,
    plan_id                     TEXT NOT NULL,
    status                      TEXT NOT NULL DEFAULT 'active',
        -- 'active' | 'cancelled' | 'past_due' | 'halted' | 'created'
    current_period_start        TIMESTAMPTZ NOT NULL,
    current_period_end          TIMESTAMPTZ NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Rate limit tracking (per-user, per-action)
CREATE TABLE rate_limits (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    action_type     TEXT NOT NULL,  -- 'generate' | 'regenerate' | 'export'
    window_start    TIMESTAMPTZ NOT NULL,
    request_count   INT NOT NULL DEFAULT 1,
    UNIQUE(user_id, action_type, window_start)
);

-- Indexes
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_refresh_token_hash ON sessions(refresh_token_hash) WHERE NOT revoked;
CREATE INDEX idx_projects_user_id ON projects(user_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_project_applicants_project_id ON project_applicants(project_id);
CREATE INDEX idx_interview_responses_project_id ON interview_responses(project_id);
CREATE INDEX idx_patent_sections_project_id ON patent_sections(project_id);
CREATE INDEX idx_section_versions_section_id ON section_versions(section_id);
CREATE INDEX idx_figures_project_id ON figures(project_id);
CREATE INDEX idx_exports_project_id ON exports(project_id);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_razorpay_sub_id ON subscriptions(razorpay_subscription_id);
CREATE INDEX idx_rate_limits_user_action ON rate_limits(user_id, action_type, window_start);
```

---

## 5. Interview Wizard Flow

### Steps

**Step 1: Basics**

- Invention title (text input)
- Patent type: Provisional or Complete (radio)
- Technical field (dropdown + custom: Mechanical, Software, Chemical, Electrical, Biotech, Other)

**Step 2: Applicant & Filing Details**

- Applicant name (text)
- Applicant address (textarea)
- Applicant nationality (dropdown, default: Indian)
- Inventor name (text, can be same as applicant)
- Inventor address (textarea)
- Inventor nationality (dropdown, default: Indian)
- Patent agent name (optional text — blank if self-filing)
- Agent registration number (optional text)
- Assignee name (optional text — if different from applicant)
- Priority claim: date, country, application number (all optional)

> **Note:** This step saves to `project_applicants` table (not `interview_responses`). This data populates the IPO Form 2 title page on export and is NOT sent to the AI pipeline.

**Step 3: Problem & Prior Art**

- What problem does this invention solve? (textarea)
- How is this problem currently solved? (textarea)
- What are the limitations of existing solutions? (textarea)

**Step 4: Invention Description**

- Describe your invention in plain language (textarea)
- What are the key components/elements? (textarea)
- How do the components work together? Step-by-step process (textarea)

**Step 5: Novelty & Advantages**

- What is new/novel about your invention vs. prior art? (textarea)
- What are the advantages/benefits? (textarea)
- Are there alternative embodiments? (optional textarea)

**Step 6: Figures (optional)**

- Upload sketches/diagrams (image upload, multiple)
- Brief description of each figure (text per image)

**Step 7: Review & Generate**

- Summary of all responses (editable, grouped by step)
- Applicant details summary (read-only, "Edit" links back to Step 2)
- "Generate Patent Draft" button
- Estimated generation time: ~60-90 seconds

### Behavior

- Steps 1 and 3-5 save to `interview_responses` on navigation (auto-save)
- Step 2 saves to `project_applicants` on navigation (auto-save)
- Step 6 saves figures via `StorageClient` + `figures` table on upload
- User can navigate back to any completed step
- Browser close/refresh resumes at last completed step
- Step 7 shows all responses for final review before generation
- "Generate" button disabled until all required fields in Steps 1-5 are complete

---

## 6. AI Drafting Pipeline

### Generation Flow

1. **Build Master Context** — combine all interview responses into a structured invention disclosure document
2. **Generate Sections Sequentially** — each section builds on previous ones:
   - a) Title
   - b) Field of Invention
   - c) Background / Prior Art
   - d) Summary of Invention
   - e) Detailed Description (longest section)
   - f) Claims (independent first, then dependent)
   - g) Abstract (summarizes claims + description)
   - h) Drawings Description (from uploaded figures or generated from text — figure descriptions from the `figures` table are included in the prompt context; images are NOT sent to the LLM for MVP, only their text descriptions)
3. **Post-Generation Validation** — two checks run after all sections are generated:
   - **Abstract word count check** — if the abstract exceeds 150 words (IPO limit), a warning suggestion is emitted immediately via `cross_reference_suggestion` event so the user sees it in the editor, not only at export time.
   - **Cross-Reference Check** — lightweight LLM call that checks claim terms vs. specification, figure references, and antecedent basis. Returns a list of **suggestions** (not auto-applied). Each suggestion includes: section, issue description, proposed fix. Surfaced to the user as a review checklist in the editor UI.
4. **Save to `patent_sections`** — each section → one row, project status → 'review'

### Streaming

- Each section streams via SSE to the frontend as it generates
- Frontend displays completed sections immediately — user reads title while claims are still generating
- Progress indicator shows which section is currently generating

### SSE Protocol

The SSE stream uses the following event types:

```
event: section_start
data: {"section_type": "title", "index": 0, "total": 8}

event: content_delta
data: {"section_type": "title", "delta": "Automated Irrigation Controller"}

event: section_complete
data: {"section_type": "title", "content": "full accumulated text"}

event: cross_reference_start
data: {}

event: cross_reference_suggestion
data: {"section_type": "claims", "issue": "Claim 1 uses 'controller' but specification uses 'control unit'", "suggestion": "Replace 'controller' with 'control unit' in Claim 1 for consistency"}

event: cross_reference_complete
data: {"suggestion_count": 3}

event: generation_complete
data: {"project_id": "uuid", "sections_generated": 8}

event: error
data: {"section_type": "claims", "message": "LLM provider timeout", "recoverable": true}
```

- `content_delta` — incremental text chunk (append to UI)
- `section_complete` — full text for the section (use as source of truth, replaces deltas)
- `error` with `recoverable: true` — frontend shows retry option; backend sets project status back to `interview_complete`

### Prompt Architecture

```
System prompt (per section type)
├── Role: "You are an Indian patent specification drafter..."
├── IPO format rules for this section type
├── Style guidelines (formal language, proper antecedent basis)
└── Output format instructions

User prompt
├── Full invention disclosure (from interview responses)
├── Previously generated sections (for context/consistency)
└── Section-specific generation instructions
```

### Section Regeneration

- User can click "Regenerate" on any individual section card
- Backend re-runs just that section's prompt with full context (interview + all other current sections)
- Confirmation dialog before overwriting ("This will replace the current content. Continue?")
- **Previous version is saved** to `section_versions` before overwriting (version_number auto-increments)
- Users can view version history and restore any previous version via the section card UI

### Error Recovery

If AI generation fails partway through (e.g., LLM provider timeout after 4 of 8 sections):

1. Already-completed sections are **kept** in `patent_sections` (they are valid)
2. Project status is set back to `interview_complete`
3. SSE stream emits an `error` event with `recoverable: true`
4. Frontend shows: "Generation failed at [section]. X sections were saved. Retry?"
5. On retry, the backend checks which sections already exist and **only generates missing sections** — it does not re-generate completed ones
6. User can also manually edit the completed sections while the remaining ones are missing

### Provider Abstraction

```rust
// Trait-based provider abstraction
trait LlmProvider {
    async fn generate_stream(&self, prompt: &Prompt) -> Result<impl Stream<Item = String>>;
}

// Implementations
struct AnthropicProvider { /* Claude */ }
struct OpenAiProvider { /* GPT - stubbed for MVP */ }

// Config-driven selection (API key only required for real providers)
let provider = match config.ai_provider.as_str() {
    "mock" => MockProvider::new(),  // no API key needed
    "openai" => OpenAiProvider::new(config.openai_api_key.expect("OPENAI_API_KEY required")),
    _ => AnthropicProvider::new(config.anthropic_api_key.expect("ANTHROPIC_API_KEY required")),
};
```

---

## 7. Section-Based Editor UI

### Layout

Each patent section is rendered as a card with:

- Section title header (e.g., "TITLE OF THE INVENTION", "CLAIMS")
- Content area (rendered text, collapsed for long sections with "Show more")
- Edit button — toggles into textarea edit mode with save/cancel
- Regenerate button — re-runs AI for this section only
- History button — shows version history panel with list of previous versions (timestamp, source), click to preview, "Restore" button to revert
- Status badge — "AI Generated" / "Edited" / "Regenerating..."

### Behavior

- **Fixed section ordering** matching IPO Form 2 — not draggable
- **Auto-save** on edit — saves to backend on blur or after 2s inactivity
- **Long sections collapsed** — Background, Detailed Description show first ~5 lines with expand toggle
- **Edit mode** — section expands to full textarea, save/cancel buttons appear
- **Regenerate** — confirmation dialog, then streams new content replacing old

### Pages

| Route                   | Purpose                                                                                                                                                                                                              |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `/login`                | Email/password login (self-managed auth, Rust backend)                                                                                                                                                               |
| `/subscribe`            | Subscription gate — shown when user has no active subscription. Displays plan details and "Subscribe" button that creates a Razorpay Subscription and opens the Razorpay checkout modal. On success, redirects to `/projects`. |
| `/projects`             | List of user's patent projects (cards: title, status, date)                                                                                                                                                          |
| `/projects/new`         | Interview wizard (Steps 1-7)                                                                                                                                                                                         |
| `/projects/[id]`        | Section-based editor                                                                                                                                                                                                 |
| `/projects/[id]/export` | Export page — format selection (PDF/DOCX buttons), shows generation progress while file is being created, lists previous exports with download links, displays error state with retry option on failure              |
| `/account`              | Profile, subscription status, manage/cancel subscription                                                                                                                                                             |

---

## 8. Export Pipeline

### PDF Generation (typst CLI)

Typst is invoked as a **CLI subprocess** (`typst compile`) — not as a Rust library. The Fly.io Docker image must include the typst binary (~50MB). The Rust backend generates a `.typ` template file from patent sections, then shells out to typst for PDF rendering. This is the most stable approach as the typst library API is not yet stable for embedding.

- IPO Form 2 compliant layout
- A4 page size with IPO-standard margins
- Title page with filing metadata from `project_applicants` (applicant name, address, nationality, inventor details, agent info, priority claims)
- Numbered paragraphs throughout specification
- Claims on separate page, numbered sequentially
- Abstract limited to 150 words (validated, warn user if over)
- "We claim:" / "I claim:" prefix before claims
- Figure references properly formatted

### DOCX Generation (docx-rs)

- Same formatting as PDF
- Editable in Microsoft Word / LibreOffice
- Styled headings for each section
- Proper claim indentation and numbering
- Attorneys can make final edits in familiar tool

### Flow

1. User clicks "Export as PDF" or "Export as DOCX"
2. Rust backend assembles all `patent_sections` in order + `project_applicants` metadata for title page
3. Generates file using typst (PDF) or docx-rs (DOCX)
4. Uploads via `StorageClient` (local filesystem in dev, Cloudflare R2 in production)
5. Saves record in `exports` table with storage key
6. Returns download URL from storage backend to frontend
7. Browser triggers download

---

## 9. Authentication & Authorization

### Auth Flow

- **Self-managed auth in Rust** — email/password login, argon2 password hashing, JWT issuance
- **No public registration for MVP** — users created via CLI seed command or admin endpoint
- Frontend stores JWT in httpOnly cookie (set by Rust backend via `Set-Cookie` header)
- Every API request to Rust backend reads JWT from httpOnly cookie (sent automatically with `credentials: "include"`)
- Rust middleware validates JWT signature against a server-side secret (HMAC-SHA256)
- User ID extracted from token claims for all database queries
- JWT expiry: 24 hours (stateless — no DB lookup on each request)
- Refresh token expiry: 7 days, **stored server-side** in `sessions` table (hashed, revocable)
- **Token type differentiation:** Access tokens use `iss: "pdp:access"`, refresh tokens use `iss: "pdp:refresh"`. The auth middleware rejects refresh tokens used as access tokens (and vice versa) by checking the `iss` claim.

### Auth Endpoints

```
POST /api/auth/login    → email + password → validates → creates session row → sets JWT + refresh token (httpOnly cookies)
POST /api/auth/refresh  → refresh token cookie → verifies iss="pdp:refresh" → looks up session (reject if revoked/expired) → issues new JWT (iss="pdp:access") + rotates refresh token (iss="pdp:refresh")
POST /api/auth/logout   → revokes session row (sets revoked=true) → clears cookies
```

### Session Security

- Refresh tokens are **hashed** (SHA-256) before storage — raw tokens only exist in cookies
- On refresh, the old refresh token is **rotated**: old session revoked, new session created
- Logout **revokes the server-side session** — a stolen refresh token becomes invalid immediately
- Expired sessions are cleaned up by a periodic `DELETE FROM sessions WHERE expires_at < now()` (cron or on-login cleanup)

### Access Gating

- **No subscription → redirected to `/subscribe`** page with plan details and Razorpay checkout
- **Active subscription → full access** to all features
- Subscription check is middleware on protected routes (projects, generate, export)
- **Performance note:** The subscription middleware hits the DB on every protected request. This is acceptable for MVP volume. Post-MVP optimization: cache subscription status in an in-memory TTL cache (e.g., `moka` crate, 60s TTL) keyed by user_id, invalidated on webhook events.
- For local development: `make seed-user` creates a test user **with an active subscription** (inserts both a `users` row and a `subscriptions` row with `status='active'` and `current_period_end` set 1 year in the future). No Razorpay interaction needed for local dev.

---

## 10. Billing (Razorpay)

### Why Razorpay

- Target users are Indian patent agents — primarily pay via UPI or Indian debit/credit cards
- Best-in-class UPI experience (auto-collect, QR, intent flow)
- Native INR pricing — no currency conversion fees
- Razorpay Subscriptions handles recurring billing
- Lower transaction fees for domestic INR payments (~2%)
- India-centric dashboard and support

### Integration Scope

| Feature                        | MVP      | Notes                               |
| ------------------------------ | -------- | ----------------------------------- |
| Razorpay Subscriptions         | Yes      | Single plan, INR pricing            |
| Razorpay Checkout (embedded)   | Yes      | Opens modal on subscribe page       |
| Manage/cancel subscription     | Yes      | In-app via Razorpay API calls       |
| Webhooks                       | Yes      | 3 events (see below)                |
| UPI + Cards + Netbanking       | Yes      | All domestic payment methods        |
| Multiple plans                 | No       | Single plan, pricing TBD            |
| Usage-based                    | No       | Post-MVP                            |

### Webhook Events

Handled by Rust backend at `POST /api/webhooks/razorpay`:

1. `subscription.activated` — new subscription active, upsert to `subscriptions` table
2. `subscription.charged` — recurring payment succeeded, update `current_period_start/end`
3. `subscription.cancelled` / `subscription.halted` — subscription ended, update status

### Webhook Handler

Lives in Rust backend (`routes/webhooks.rs`) — verifies Razorpay webhook signature (HMAC-SHA256 using webhook secret), updates database via sqlx.

**Important:** All writes to `subscriptions` use **upsert** (`INSERT ... ON CONFLICT (razorpay_subscription_id) DO UPDATE`) to handle webhook retries idempotently. The `razorpay_subscription_id UNIQUE` constraint serves as the conflict target. For `subscription.charged` events, the period dates are only updated if the incoming `current_period_end` is **newer** than the existing value (`WHERE current_period_end < EXCLUDED.current_period_end`), preventing stale webhook retries from overwriting newer state.

### Subscription Flow

```
1. User clicks "Subscribe" on /subscribe page
2. Frontend calls POST /api/subscriptions/create (Rust backend)
3. Backend creates Razorpay Subscription via API, returns subscription_id
4. Frontend opens Razorpay Checkout modal with subscription_id
5. User pays via UPI/Card/Netbanking
6. Razorpay sends webhook → Rust backend upserts subscription
7. Frontend polls GET /api/me until has_active_subscription = true
8. Redirect to /projects
```

---

## 11. Monorepo Structure

```
patent-draft-pro/
├── apps/
│   └── web/                          ← Next.js 16 (App Router)
│       ├── app/                      ← Pages and routes
│       ├── components/               ← UI components
│       │   ├── ui/                   ← shadcn/ui primitives
│       │   ├── interview/            ← Wizard step components
│       │   ├── editor/               ← Section card components
│       │   └── layout/               ← Navbar, auth guard
│       ├── lib/                      ← Utilities
│       │   ├── api-client.ts         ← Typed HTTP client for Rust backend
│       │   └── razorpay.ts           ← Razorpay checkout helpers
│       ├── next.config.ts
│       ├── tailwind.config.ts
│       └── package.json
│
├── backend/                          ← Rust Cargo workspace
│   ├── Cargo.toml                    ← Workspace root
│   ├── api/                          ← Axum HTTP server
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/               ← Route handlers (incl. auth + webhooks)
│   │       ├── middleware/            ← Auth (JWT), subscription, rate limiting
│   │       └── error.rs
│   ├── ai/                           ← AI service crate
│   │   └── src/
│   │       ├── provider.rs           ← Provider trait + implementations
│   │       ├── prompts/              ← System prompts per section type
│   │       └── pipeline.rs           ← Multi-section generation orchestrator
│   ├── export/                       ← Export service crate
│   │   └── src/
│   │       ├── pdf.rs                ← typst PDF generation
│   │       ├── docx.rs               ← docx-rs Word generation
│   │       └── templates/            ← IPO Form 2 templates
│   ├── storage/                      ← Storage client crate (local + R2)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs             ← StorageClient trait + factory
│   │       ├── local.rs              ← LocalStorage (filesystem, for dev)
│   │       └── r2.rs                 ← R2Storage (S3-compatible, for prod)
│   └── shared/                       ← Common crate
│       └── src/
│           ├── models.rs             ← DB models (sqlx FromRow)
│           ├── db.rs                 ← Connection pool
│           └── config.rs             ← Environment config
│
├── migrations/                       ← sqlx migrations
├── .github/workflows/                ← CI/CD (frontend + backend)
├── docker-compose.yml                ← Local dev (Postgres)
├── Makefile                          ← Dev commands
└── .env.example
```

---

## 12. Tech Stack Summary

| Layer                 | Technology                  | Purpose                          |
| --------------------- | --------------------------- | -------------------------------- |
| Frontend              | Next.js 16 (App Router)     | Pages, routing, SSR              |
| UI                    | shadcn/ui + Tailwind CSS    | Design system                    |
| Font                  | Geist Sans + Geist Mono     | Typography                       |
| AI streaming (client) | Custom SSE hook             | Stream AI text to UI             |
| Backend               | Rust + Axum                 | API server                       |
| Database access       | sqlx (compile-time checked) | Type-safe Postgres queries       |
| Database              | Postgres (Docker local, Supabase prod) | Managed Postgres (DB only) |
| Auth                  | Self-managed (argon2 + JWT) | Email/password, JWT issuance     |
| File storage          | Local FS (dev) / Cloudflare R2 (prod) | Exported files, uploaded figures |
| AI provider (default) | Anthropic Claude            | Patent text generation           |
| AI abstraction        | Custom Rust trait            | Provider-agnostic                |
| PDF export            | typst                       | IPO-formatted PDF                |
| DOCX export           | docx-rs                     | Editable Word files              |
| Billing               | Razorpay                    | Subscriptions, UPI + cards       |
| Frontend hosting      | Vercel                      | CDN + edge                       |
| Backend hosting       | Fly.io (Mumbai)             | Rust containers                  |
| CI/CD                 | GitHub Actions              | Build + deploy                   |

---

## 13. API Endpoints

### Rust Backend API

| Method   | Path                                              | Purpose                                                    | Auth | Subscription |
| -------- | ------------------------------------------------- | ---------------------------------------------------------- | ---- | ------------ |
| `GET`    | `/api/health`                                     | Health check                                               | No   | No           |
| `POST`   | `/api/auth/login`                                 | Email/password login → JWT + refresh token (cookies)       | No   | No           |
| `POST`   | `/api/auth/refresh`                               | Refresh JWT using refresh token cookie                     | No   | No           |
| `POST`   | `/api/auth/logout`                                | Clear auth cookies                                         | No   | No           |
| `GET`    | `/api/me`                                         | Get current user profile + subscription status             | Yes  | No           |
| `POST`   | `/api/subscriptions/create`                       | Create Razorpay Subscription, return subscription_id       | Yes  | No           |
| `POST`   | `/api/subscriptions/cancel`                       | Cancel active subscription via Razorpay API                | Yes  | No           |
| `GET`    | `/api/subscriptions/status`                       | Get current subscription details                           | Yes  | No           |
| `POST`   | `/api/webhooks/razorpay`                          | Razorpay webhook handler (signature verified)              | No*  | No           |
| `GET`    | `/api/projects`                                   | List user's projects                                       | Yes  | Yes          |
| `POST`   | `/api/projects`                                   | Create new project                                         | Yes  | Yes          |
| `GET`    | `/api/projects/:id`                               | Get project with sections                                  | Yes  | Yes          |
| `PATCH`  | `/api/projects/:id`                               | Update project metadata (title, patent_type)               | Yes  | Yes          |
| `DELETE` | `/api/projects/:id`                               | Soft-delete project (sets deleted_at)                      | Yes  | Yes          |
| `PUT`    | `/api/projects/:id/applicant`                     | Upsert applicant/inventor/agent details                    | Yes  | Yes          |
| `GET`    | `/api/projects/:id/applicant`                     | Get applicant details for a project                        | Yes  | Yes          |
| `PUT`    | `/api/projects/:id/interview`                     | Save interview responses (batch)                           | Yes  | Yes          |
| `GET`    | `/api/projects/:id/interview`                     | Get interview responses                                    | Yes  | Yes          |
| `POST`   | `/api/projects/:id/figures`                       | Upload figure image (multipart)                            | Yes  | Yes          |
| `DELETE` | `/api/projects/:id/figures/:figure_id`            | Delete a figure                                            | Yes  | Yes          |
| `GET`    | `/api/projects/:id/figures`                       | List figures for project                                   | Yes  | Yes          |
| `POST`   | `/api/projects/:id/generate`                      | Start AI generation (returns SSE stream) — rate limited    | Yes  | Yes          |
| `PUT`    | `/api/projects/:id/sections/:type`                | Update section content (manual edit)                       | Yes  | Yes          |
| `POST`   | `/api/projects/:id/sections/:type/regenerate`     | Regenerate single section (SSE stream) — rate limited      | Yes  | Yes          |
| `GET`    | `/api/projects/:id/sections/:type/versions`       | List version history for a section                         | Yes  | Yes          |
| `POST`   | `/api/projects/:id/sections/:type/versions/:ver/restore` | Restore a previous version                          | Yes  | Yes          |
| `POST`   | `/api/projects/:id/export`                        | Generate export (PDF or DOCX) — rate limited               | Yes  | Yes          |
| `GET`    | `/api/projects/:id/exports`                       | List exports for project                                   | Yes  | Yes          |
| `GET`    | `/api/exports/:id/download`                       | Get download URL from storage backend                      | Yes  | Yes          |

- **Auth** = valid JWT in Authorization header (or httpOnly cookie) required
- **Subscription** = active subscription required (middleware check)
- **No*** = Razorpay webhook uses signature verification instead of JWT auth
- `/api/me` and `/api/subscriptions/*` do NOT require subscription — needed for the subscribe flow before payment
- **Rate limited** endpoints: generate (5/hour), regenerate (20/hour), export (10/hour) per user

### Rate Limiting

Two layers of rate limiting:

**1. Application-level (Rust middleware)** — per-user limits on expensive operations:

| Action       | Limit       | Window | Enforcement          |
| ------------ | ----------- | ------ | -------------------- |
| `generate`   | 5 requests  | 1 hour | 429 Too Many Requests |
| `regenerate` | 20 requests | 1 hour | 429 Too Many Requests |
| `export`     | 10 requests | 1 hour | 429 Too Many Requests |

Tracked in `rate_limits` table. Middleware checks before executing the operation.

**2. Edge-level (Vercel Firewall)** — protects frontend from abuse:

| Rule                    | Limit          | Window | Action    |
| ----------------------- | -------------- | ------ | --------- |
| API routes              | 100 req/min    | 60s    | deny (429)|
| Login page              | 10 req/min     | 60s    | challenge |

---

## 14. Environment Variables

### Frontend (.env.local)

```
NEXT_PUBLIC_API_URL=http://localhost:5012  # Rust backend URL
NEXT_PUBLIC_RAZORPAY_KEY_ID=rzp_test_...  # Razorpay publishable key
```

### Backend (.env)

```
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/patent_draft_pro
JWT_SECRET=your-random-64-char-secret
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...                    # optional, for future use
AI_PROVIDER=mock                          # "anthropic" | "mock" (use mock for local dev)

# Storage backend: "local" for development, "r2" for production
STORAGE_BACKEND=local                     # "local" | "r2"
STORAGE_LOCAL_PATH=./storage              # local dev: files saved here (auto-created)

# Cloudflare R2 (only needed when STORAGE_BACKEND=r2)
R2_ACCOUNT_ID=your-cf-account-id
R2_ACCESS_KEY_ID=your-r2-access-key
R2_SECRET_ACCESS_KEY=your-r2-secret-key
R2_BUCKET_NAME=patent-draft-pro
R2_PUBLIC_URL=https://files.patentdraftpro.com

# Razorpay
RAZORPAY_KEY_ID=rzp_test_...
RAZORPAY_KEY_SECRET=your-razorpay-secret
RAZORPAY_WEBHOOK_SECRET=your-webhook-secret
RAZORPAY_PLAN_ID=plan_...                # subscription plan ID

RUST_LOG=info
PORT=5012
ALLOWED_ORIGIN=http://localhost:3000   # Lock CORS to frontend origin (use production URL in prod)
```

---

## 15. Testing Strategy

### Overview

| Layer                          | Framework                          | Scope                                                            | Runs In    |
| ------------------------------ | ---------------------------------- | ---------------------------------------------------------------- | ---------- |
| **Rust unit tests**            | `cargo test` (built-in)            | Pure functions, models, prompt building, export formatting       | CI + local |
| **Rust integration tests**     | `cargo test` + testcontainers      | API routes against real Postgres, auth middleware, SSE streaming | CI + local |
| **Frontend unit tests**        | Vitest + React Testing Library     | Components, hooks, form validation, API client                   | CI + local |
| **Frontend integration tests** | Vitest + MSW (Mock Service Worker) | Page-level flows with mocked API responses                       | CI + local |
| **E2E tests**                  | Playwright                         | Full user flows: login → interview → generate → edit → export    | CI + local |

### Rust Unit Tests

Located alongside source files (`#[cfg(test)]` modules).

**shared crate:**

- `models.rs` — serialization/deserialization of all DB models
- `config.rs` — environment variable parsing, defaults, validation

**ai crate:**

- `pipeline.rs` — prompt context assembly from interview responses (given structured input, produces correct prompt string)
- `pipeline.rs` — section ordering logic (correct sequential order)
- `pipeline.rs` — cross-reference pass prompt construction
- `provider.rs` — provider selection from config string
- `prompts/` — each section prompt template renders correctly with sample data (title, claims, background, etc.)

**export crate:**

- `pdf.rs` — typst template generation from patent sections (produces valid `.typ` file content)
- `pdf.rs` — abstract word count validation (>150 words triggers warning)
- `pdf.rs` — claim numbering and formatting
- `docx.rs` — DOCX structure generation (correct heading styles, section ordering)
- `docx.rs` — IPO Form 2 metadata page content

**api crate:**

- `routes/auth.rs` — JWT claim extraction (valid token → user_id, expired token → error, malformed → error)
- `routes/interview.rs` — interview response validation (required fields, max lengths)
- `routes/sections.rs` — section type validation (only valid IPO section types accepted)
- `routes/export.rs` — export format validation
- `middleware/subscription.rs` — subscription status check logic (active → pass, expired → reject, missing → reject)
- `error.rs` — error response formatting (all error types produce correct HTTP status + JSON body)

### Rust Integration Tests

Located in `backend/tests/` directory. Use `testcontainers` crate to spin up a real Postgres instance per test suite.

**Auth flow:**

- `test_login_creates_session` — POST /api/auth/login with valid credentials creates user session and returns JWT
- `test_get_me_with_valid_jwt` — GET /api/me with self-issued JWT returns user profile
- `test_reject_invalid_jwt` — request with expired/invalid JWT returns 401
- `test_reject_missing_jwt` — request without Authorization header returns 401

**Subscription middleware:**

- `test_active_subscription_allows_access` — user with active sub can access /api/projects
- `test_no_subscription_blocks_access` — user without sub gets 403 on protected routes
- `test_expired_subscription_blocks_access` — past_due/canceled sub returns 403
- `test_me_endpoint_bypasses_subscription` — /api/me works without subscription

**Projects CRUD:**

- `test_create_project` — POST /api/projects creates project with correct defaults
- `test_list_projects` — GET /api/projects returns only current user's projects (multi-tenant isolation)
- `test_get_project_with_sections` — GET /api/projects/:id includes patent_sections
- `test_patch_project` — PATCH /api/projects/:id updates title and patent_type
- `test_delete_project_soft_deletes` — DELETE sets `deleted_at`, project no longer appears in list, but data is preserved in DB
- `test_cannot_access_other_users_project` — GET /api/projects/:other_user_id returns 404

**Interview responses:**

- `test_save_interview_batch` — PUT /api/projects/:id/interview saves all responses
- `test_save_interview_upsert` — saving again updates existing responses (not duplicates)
- `test_get_interview_responses` — returns responses in step_number order

**Figures:**

- `test_upload_figure` — POST multipart upload stores file and creates DB record
- `test_list_figures` — returns figures in sort_order
- `test_delete_figure` — removes DB record and storage file

**AI generation (with mocked LLM):**

- `test_generate_streams_sse_events` — POST /api/projects/:id/generate returns SSE stream with correct event types (section_start, content_delta, section_complete, generation_complete)
- `test_generate_saves_all_sections` — after generation completes, all 8 sections exist in DB
- `test_generate_sets_project_status` — project status changes: interview_complete → generating → review
- `test_generate_partial_failure_recovery` — simulate LLM failure at section 4: first 3 sections saved, project status reset to interview_complete, error event emitted
- `test_regenerate_single_section` — POST /api/projects/:id/sections/claims/regenerate updates only claims section
- `test_regenerate_increments_edit_count` — edit_count increases on regeneration

**Section editing:**

- `test_update_section_content` — PUT /api/projects/:id/sections/title updates content
- `test_update_section_sets_ai_generated_false` — manual edit marks ai_generated = false
- `test_update_section_increments_edit_count` — edit_count increases on manual edit
- `test_update_invalid_section_type` — invalid section_type returns 400

**Export:**

- `test_export_pdf` — POST /api/projects/:id/export with format=pdf generates file, stores via `StorageClient` (LocalStorage in tests), creates exports row
- `test_export_docx` — same for DOCX
- `test_export_download_url` — GET /api/exports/:id/download returns download URL (presigned URL for R2, local file URL for LocalStorage)
- `test_export_requires_all_sections` — export with missing sections returns 400 with list of missing sections

### Frontend Unit Tests (Vitest + React Testing Library)

**Components:**

- `StepBasics.test.tsx` — renders title, patent type, technical field inputs; validates required fields
- `StepApplicant.test.tsx` — renders applicant/inventor/agent fields; saves to project_applicants; optional fields can be empty
- `StepProblem.test.tsx` — renders problem/prior art textareas; validates required fields
- `StepDescription.test.tsx` — renders invention description textareas; validates required fields
- `StepNovelty.test.tsx` — renders novelty/advantages textareas; optional embodiments field can be empty
- `StepFigures.test.tsx` — renders file upload, handles multiple images, figure descriptions
- `StepReview.test.tsx` — renders all responses + applicant summary, edit buttons navigate to correct step, generate button disabled when fields missing
- `SectionCard.test.tsx` — renders section content, toggles edit mode, shows status badge
- `SectionCard.test.tsx` — edit mode: textarea, save/cancel buttons, auto-save triggers
- `SectionCard.test.tsx` — regenerate button shows confirmation dialog
- `SectionCard.test.tsx` — collapsed state shows first 5 lines + "Show more"
- `ProjectCard.test.tsx` — renders title, status, date, click navigates
- `ExportPage.test.tsx` — renders format buttons, shows progress, lists previous exports
- `SubscribePage.test.tsx` — renders plan details, subscribe button opens Razorpay checkout modal
- `AccountPage.test.tsx` — renders profile, subscription status, manage billing link

**Hooks:**

- `useInterviewWizard.test.ts` — step navigation, data persistence, resume from last step
- `useSSEStream.test.ts` — parses SSE events correctly (section_start, content_delta, section_complete, error)
- `useAutoSave.test.ts` — debounces saves, triggers on blur

**Lib:**

- `api-client.test.ts` — constructs correct URLs, attaches auth header, handles 401/403/500 responses
- `api-client.test.ts` — retry logic on network errors

### Frontend Integration Tests (Vitest + MSW)

Page-level tests with mocked API responses via Mock Service Worker.

- `LoginPage.test.tsx` — login form submits to Rust backend (`POST /api/auth/login`), redirects to /projects on success, shows error on failure
- `ProjectsPage.test.tsx` — fetches and renders project list, empty state shows "Create your first patent draft" CTA
- `NewProjectPage.test.tsx` — full wizard flow: Step 1 → 2 → 3 → 4 → 5 → 6 → 7, saves responses at each step, generate button triggers API call
- `EditorPage.test.tsx` — loads project with sections, renders all section cards, edit saves to API, regenerate streams SSE
- `EditorPage.test.tsx` — generation in progress: shows streaming sections with progress indicator
- `ExportPage.test.tsx` — triggers export, shows progress, handles download URL response
- `SubscribePage.test.tsx` — subscribe flow calls createSubscription API, opens Razorpay modal
- `AccountPage.test.tsx` — loads user profile + subscription, manage/cancel subscription via API

### E2E Tests (Playwright)

Full browser tests against running frontend + backend (backend uses test database, LLM calls use mock provider).

**Test setup:**

- Docker Compose spins up test Postgres + Rust backend with mock LLM provider
- Next.js dev server runs against test backend
- Seed data: one test user with active subscription
- Mock LLM returns predetermined patent text (fast, deterministic)

**Auth flows:**

- `auth.spec.ts` — login with valid credentials → redirects to /projects
- `auth.spec.ts` — login with invalid credentials → shows error message
- `auth.spec.ts` — unauthenticated access to /projects → redirects to /login
- `auth.spec.ts` — authenticated user without subscription → redirects to /subscribe

**Interview wizard:**

- `interview.spec.ts` — complete full wizard (Steps 1-7) with valid data → all fields saved
- `interview.spec.ts` — navigate back and forth between steps → data persists
- `interview.spec.ts` — refresh browser mid-wizard → resumes at correct step with data intact
- `interview.spec.ts` — skip optional fields (Step 5 alternative embodiments, Step 6 figures) → proceeds normally
- `interview.spec.ts` — try to proceed with empty required fields → validation error shown

**AI generation:**

- `generation.spec.ts` — click "Generate" on Step 7 → sections appear one by one with streaming animation
- `generation.spec.ts` — all 8 sections render in correct IPO order
- `generation.spec.ts` — project status updates to "In Review" after generation completes
- `generation.spec.ts` — navigate to /projects → project shows "In Review" status

**Editor:**

- `editor.spec.ts` — click edit on a section → textarea appears with content
- `editor.spec.ts` — modify text + click save → content updates, status badge shows "Edited"
- `editor.spec.ts` — click cancel in edit mode → reverts to original content
- `editor.spec.ts` — click regenerate → confirmation dialog → new content streams in
- `editor.spec.ts` — long section (Detailed Description) → shows collapsed with "Show more" → click expands
- `editor.spec.ts` — auto-save: edit text, wait 2s → saved without clicking save button

**Export:**

- `export.spec.ts` — click "Export as PDF" → progress indicator → download triggers
- `export.spec.ts` — click "Export as DOCX" → progress indicator → download triggers
- `export.spec.ts` — export page lists previous exports with download links
- `export.spec.ts` — downloaded PDF is valid (non-zero size, correct content type)

**Subscription:**

- `subscription.spec.ts` — user without subscription sees /subscribe page with plan details
- `subscription.spec.ts` — click "Subscribe" → Razorpay Checkout modal opens (verify modal loads)
- `subscription.spec.ts` — after successful subscription → can access /projects

**Projects:**

- `projects.spec.ts` — empty state: shows "Create your first patent draft" with CTA
- `projects.spec.ts` — create project → appears in list
- `projects.spec.ts` — delete project → removed from list (with confirmation dialog)
- `projects.spec.ts` — multiple projects → sorted by most recent

**Multi-tenant isolation (security):**

- `isolation.spec.ts` — user A cannot see user B's projects (API returns 404)
- `isolation.spec.ts` — user A cannot access user B's project editor via direct URL

### CI Pipeline (GitHub Actions)

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  rust-unit:
    runs-on: ubuntu-latest
    steps:
      - cargo test --workspace --lib

  rust-integration:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
    steps:
      - cargo test --workspace --test '*'

  frontend-unit:
    runs-on: ubuntu-latest
    steps:
      - npm ci
      - npm run test # vitest unit + integration

  e2e:
    runs-on: ubuntu-latest
    needs: [rust-unit, frontend-unit] # only run E2E if unit tests pass
    steps:
      - docker-compose up -d # postgres + rust backend (mock LLM)
      - npm ci
      - npx playwright install --with-deps chromium
      - npm run test:e2e # playwright
```

### Test Infrastructure

| Component                | Purpose                                                                                |
| ------------------------ | -------------------------------------------------------------------------------------- |
| `testcontainers` (Rust)  | Ephemeral Postgres per test suite — no shared state, no cleanup                        |
| `MockLlmProvider` (Rust) | Implements `LlmProvider` trait, returns predetermined text. Used in integration + E2E. |
| MSW (Frontend)           | Intercepts HTTP requests in frontend tests, returns mock API responses                 |
| Docker Compose (E2E)     | Spins up full backend stack with test database and mock LLM                            |
| Playwright fixtures      | Shared test user login, project seeding, subscription setup                            |

### Test Coverage Targets

| Layer                | Target              | Rationale                                                                       |
| -------------------- | ------------------- | ------------------------------------------------------------------------------- |
| Rust unit            | >90%                | Core business logic — prompt assembly, export formatting, validation            |
| Rust integration     | All API endpoints   | Every endpoint has at least happy path + auth failure + validation failure test |
| Frontend unit        | >80%                | All components rendered, all interactive states tested                          |
| Frontend integration | All pages           | Each page has full flow test with mocked API                                    |
| E2E                  | Critical user paths | Login, wizard, generate, edit, export — the core product loop                   |

---

## 16. Post-MVP Roadmap

### Phase 2: Additional Modules

- Office Action Response (analyze examiner rejections, generate responses)
- Prior Art Search (AI-powered search across patent databases)
- Invention Disclosure Intake (guided intake for inventors, feeds into drafting)

### Phase 3: Jurisdictions

- PCT National Phase entry for India
- US (USPTO) format support
- EPO format support

### Phase 4: Multi-User & Teams

- Team accounts with shared workspaces
- Role-based access (admin, drafter, reviewer)
- Collaboration features (comments, review workflow)
- Public self-service registration

### Phase 5: Platform

- Admin dashboard and analytics
- Marketing/landing page
- Usage-based billing options
- Direct IPO e-filing integration
- Section diff view (compare versions side-by-side)
- Stripe integration for international customers
