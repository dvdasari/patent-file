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
- Email/password authentication (private — no public signup for MVP)
- Stripe subscription billing (single plan, pricing TBD by client)
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
│              Next.js 15 (App Router)                     │
│              Vercel (free tier)                           │
│                                                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Login   │  │  Interview   │  │  Draft Editor     │  │
│  │  Page    │  │  Wizard      │  │  (Section Cards)  │  │
│  └──────────┘  └──────────────┘  └──────────────────┘  │
│                        │                    │            │
│              AI SDK (streaming)     REST API calls       │
└────────────────────────┼────────────────────┼───────────┘
                         │                    │
                    HTTPS (JSON + SSE)
                         │                    │
┌────────────────────────┼────────────────────┼───────────┐
│                      BACKEND                             │
│                 Rust (Axum)                               │
│                 Fly.io                                    │
│                                                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Auth    │  │  AI Service  │  │  Export Service   │  │
│  │  (JWT)   │  │  (LLM calls) │  │  (PDF/DOCX gen)  │  │
│  └──────────┘  └──────────────┘  └──────────────────┘  │
│                        │                                 │
│                   ┌────┴────┐                            │
│                   │  sqlx   │                            │
│                   └────┬────┘                            │
└────────────────────────┼────────────────────────────────┘
                         │
              ┌──────────┴──────────┐
              │  Supabase           │
              │  - Postgres (DB)    │
              │  - Auth (JWT)       │
              │  - Storage (files)  │
              └─────────────────────┘
```

### Key Architectural Decisions

1. **AI calls go through Rust backend** — not directly from frontend. Controls rate limiting, prompt engineering, and token usage server-side.
2. **Streaming via SSE** — AI-generated text streams from Rust → Next.js frontend so users see text appear in real-time.
3. **Supabase for managed services** — Auth (JWT issuance), Postgres (data), Storage (exported files). Rust connects to Postgres directly via sqlx.
4. **Provider-agnostic AI layer** — trait-based abstraction in Rust. Start with Claude (Anthropic), stub OpenAI. Swap or A/B test providers via config.
5. **Stripe webhook on Next.js** — webhook handler lives in Next.js API route (Vercel serverless function) since it's lightweight and avoids exposing the Rust backend to Stripe directly.

---

## 4. Database Schema

```sql
-- Users (mirrors Supabase Auth key fields)
CREATE TABLE users (
    id              UUID PRIMARY KEY,  -- from Supabase Auth
    email           TEXT UNIQUE NOT NULL,
    full_name       TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Patent projects (one per invention)
CREATE TABLE projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'draft',
        -- 'draft' | 'interview_complete' | 'generating' | 'review' | 'exported'
    jurisdiction    TEXT NOT NULL DEFAULT 'IPO',
    patent_type     TEXT NOT NULL DEFAULT 'complete',
        -- 'provisional' | 'complete'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
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

-- Figures (uploaded invention sketches/diagrams)
CREATE TABLE figures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    sort_order      INT NOT NULL DEFAULT 0,
    description     TEXT NOT NULL,  -- user-provided figure description
    storage_path    TEXT NOT NULL,  -- Supabase Storage path
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
    storage_path    TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Subscriptions (Stripe state cached locally)
CREATE TABLE subscriptions (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                  UUID NOT NULL REFERENCES users(id),
    stripe_customer_id       TEXT NOT NULL,
    stripe_subscription_id   TEXT NOT NULL UNIQUE,
    plan_id                  TEXT NOT NULL,
    status                   TEXT NOT NULL DEFAULT 'active',
        -- 'active' | 'canceled' | 'past_due' | 'trialing'
    current_period_start     TIMESTAMPTZ NOT NULL,
    current_period_end       TIMESTAMPTZ NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX idx_projects_user_id ON projects(user_id);
CREATE INDEX idx_interview_responses_project_id ON interview_responses(project_id);
CREATE INDEX idx_patent_sections_project_id ON patent_sections(project_id);
CREATE INDEX idx_figures_project_id ON figures(project_id);
CREATE INDEX idx_exports_project_id ON exports(project_id);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_stripe_subscription_id ON subscriptions(stripe_subscription_id);
```

---

## 5. Interview Wizard Flow

### Steps

**Step 1: Basics**

- Invention title (text input)
- Patent type: Provisional or Complete (radio)
- Technical field (dropdown + custom: Mechanical, Software, Chemical, Electrical, Biotech, Other)

**Step 2: Problem & Prior Art**

- What problem does this invention solve? (textarea)
- How is this problem currently solved? (textarea)
- What are the limitations of existing solutions? (textarea)

**Step 3: Invention Description**

- Describe your invention in plain language (textarea)
- What are the key components/elements? (textarea)
- How do the components work together? Step-by-step process (textarea)

**Step 4: Novelty & Advantages**

- What is new/novel about your invention vs. prior art? (textarea)
- What are the advantages/benefits? (textarea)
- Are there alternative embodiments? (optional textarea)

**Step 5: Figures (optional)**

- Upload sketches/diagrams (image upload, multiple)
- Brief description of each figure (text per image)

**Step 6: Review & Generate**

- Summary of all responses (editable)
- "Generate Patent Draft" button
- Estimated generation time: ~60-90 seconds

### Behavior

- Each step saves to `interview_responses` on navigation (auto-save)
- User can navigate back to any completed step
- Browser close/refresh resumes at last completed step
- Step 6 shows all responses for final review before generation
- "Generate" button disabled until all required fields in Steps 1-4 are complete

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
3. **Cross-Reference Pass** — lightweight LLM call to ensure claim terms match specification, figure references are consistent, antecedent basis is correct
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
- Previous version is not stored for MVP (edit_count increments but old content is overwritten)

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

// Config-driven selection
let provider = match config.ai_provider {
    "anthropic" => AnthropicProvider::new(config.anthropic_api_key),
    "openai" => OpenAiProvider::new(config.openai_api_key),
    _ => AnthropicProvider::new(config.anthropic_api_key), // default
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
| `/login`                | Email/password login (Supabase Auth)                                                                                                                                                                                 |
| `/subscribe`            | Subscription gate — shown when user has no active subscription. Displays plan details and "Subscribe" button that creates a Stripe Checkout session and redirects. On success, Stripe redirects back to `/projects`. |
| `/projects`             | List of user's patent projects (cards: title, status, date)                                                                                                                                                          |
| `/projects/new`         | Interview wizard (Steps 1-6)                                                                                                                                                                                         |
| `/projects/[id]`        | Section-based editor                                                                                                                                                                                                 |
| `/projects/[id]/export` | Export page — format selection (PDF/DOCX buttons), shows generation progress while file is being created, lists previous exports with download links, displays error state with retry option on failure              |
| `/account`              | Profile, subscription status, Stripe billing portal link (manage/cancel)                                                                                                                                             |

---

## 8. Export Pipeline

### PDF Generation (typst CLI)

Typst is invoked as a **CLI subprocess** (`typst compile`) — not as a Rust library. The Fly.io Docker image must include the typst binary (~50MB). The Rust backend generates a `.typ` template file from patent sections, then shells out to typst for PDF rendering. This is the most stable approach as the typst library API is not yet stable for embedding.

- IPO Form 2 compliant layout
- A4 page size with IPO-standard margins
- Title page with application metadata
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
2. Rust backend assembles all `patent_sections` in order
3. Generates file using typst (PDF) or docx-rs (DOCX)
4. Uploads to Supabase Storage
5. Saves record in `exports` table
6. Returns signed download URL to frontend
7. Browser triggers download

---

## 9. Authentication & Authorization

### Auth Flow

- **Supabase Auth** handles email/password signup, login, JWT issuance
- **No public registration for MVP** — users created manually in Supabase dashboard
- Frontend stores JWT in httpOnly cookie (Supabase client handles this)
- Every API request to Rust backend includes JWT in Authorization header
- Rust middleware validates JWT signature against Supabase JWT secret
- User ID extracted from token claims for all database queries

### User Record Creation

The `users` table row is created via a **`POST /api/me` upsert endpoint** called on first login. The Next.js frontend calls this endpoint after Supabase Auth login succeeds, before routing to `/projects` or `/subscribe`. This ensures the `users` row exists before any Stripe webhook fires, preventing the foreign key race condition.

```
Login → Supabase Auth (JWT) → POST /api/me (upsert user row) → Route to /projects or /subscribe
```

### Access Gating

- **No subscription → redirected to `/subscribe`** page with plan details and Stripe Checkout button
- **Active subscription → full access** to all features
- Subscription check is middleware on protected routes (projects, generate, export)
- For MVP development: first user can be manually set to 'active' in DB

---

## 10. Billing (Stripe)

### Integration Scope

| Feature                  | MVP      | Notes                      |
| ------------------------ | -------- | -------------------------- |
| Stripe Checkout (hosted) | Yes      | Single subscription plan   |
| Customer Portal (hosted) | Yes      | Manage/cancel subscription |
| Webhooks                 | Yes      | 3 events (see below)       |
| Multiple plans           | No       | Single plan, pricing TBD   |
| Usage-based              | No       | Post-MVP                   |
| Free trial               | Optional | Stripe-native trial period |

### Webhook Events

1. `checkout.session.completed` — new subscription created, upsert to `subscriptions` table
2. `customer.subscription.updated` — plan change, status change (active → past_due)
3. `customer.subscription.deleted` — subscription canceled, update status

### Webhook Handler

Lives in Next.js API route (`/api/webhooks/stripe/route.ts`) — lightweight, verifies Stripe signature, updates Supabase database directly via Supabase client.

**Important:** All writes to `subscriptions` use **upsert** (`INSERT ... ON CONFLICT (stripe_subscription_id) DO UPDATE`) to handle Stripe webhook retries idempotently. The `stripe_subscription_id UNIQUE` constraint serves as the conflict target.

---

## 11. Monorepo Structure

```
patent-draft-pro/
├── apps/
│   └── web/                          ← Next.js 15 (App Router)
│       ├── app/                      ← Pages and routes
│       ├── components/               ← UI components
│       │   ├── ui/                   ← shadcn/ui primitives
│       │   ├── interview/            ← Wizard step components
│       │   ├── editor/               ← Section card components
│       │   └── layout/               ← Navbar, auth guard
│       ├── lib/                      ← Utilities
│       │   ├── api-client.ts         ← Typed HTTP client for Rust backend
│       │   ├── supabase.ts           ← Supabase client config
│       │   └── stripe.ts             ← Stripe helpers
│       ├── next.config.ts
│       ├── tailwind.config.ts
│       └── package.json
│
├── backend/                          ← Rust Cargo workspace
│   ├── Cargo.toml                    ← Workspace root
│   ├── api/                          ← Axum HTTP server
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/               ← Route handlers
│   │       ├── middleware/            ← Auth, subscription checks
│   │       └── error.rs
│   ├── ai/                           ← AI service crate
│   │   └── src/
│   │       ├── provider.rs           ← Provider trait + implementations
│   │       ├── prompts/              ← System prompts per section type
│   │       └── pipeline.rs           ← Multi-section generation orchestrator
│   ├── export/                       ← Export service crate
│   │   └── src/
│   │       ├── pdf.rs                ← typst PDF generation
│   │       ├── docx.rs              ← docx-rs Word generation
│   │       └── templates/            ← IPO Form 2 templates
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
| Frontend              | Next.js 15 (App Router)     | Pages, routing, SSR              |
| UI                    | shadcn/ui + Tailwind CSS    | Design system                    |
| Font                  | Geist Sans + Geist Mono     | Typography                       |
| AI streaming (client) | AI SDK or custom SSE        | Stream AI text to UI             |
| Backend               | Rust + Axum                 | API server                       |
| Database access       | sqlx (compile-time checked) | Type-safe Postgres queries       |
| Database              | Supabase Postgres           | Managed Postgres                 |
| Auth                  | Supabase Auth               | Email/password, JWT              |
| File storage          | Supabase Storage            | Exported files, uploaded figures |
| AI provider (default) | Anthropic Claude            | Patent text generation           |
| AI abstraction        | Custom Rust trait           | Provider-agnostic                |
| PDF export            | typst                       | IPO-formatted PDF                |
| DOCX export           | docx-rs                     | Editable Word files              |
| Billing               | Stripe                      | Subscriptions, checkout          |
| Frontend hosting      | Vercel (free tier → Pro)    | CDN + serverless                 |
| Backend hosting       | Fly.io                      | Rust containers                  |
| CI/CD                 | GitHub Actions              | Build + deploy                   |

---

## 13. API Endpoints

### Rust Backend API

| Method   | Path                                          | Purpose                                                    | Auth | Subscription |
| -------- | --------------------------------------------- | ---------------------------------------------------------- | ---- | ------------ |
| `GET`    | `/api/health`                                 | Health check                                               | No   | No           |
| `POST`   | `/api/me`                                     | Upsert user record from JWT claims (called on first login) | Yes  | No           |
| `GET`    | `/api/me`                                     | Get current user profile + subscription status             | Yes  | No           |
| `GET`    | `/api/projects`                               | List user's projects                                       | Yes  | Yes          |
| `POST`   | `/api/projects`                               | Create new project                                         | Yes  | Yes          |
| `GET`    | `/api/projects/:id`                           | Get project with sections                                  | Yes  | Yes          |
| `PATCH`  | `/api/projects/:id`                           | Update project metadata (title, patent_type)               | Yes  | Yes          |
| `DELETE` | `/api/projects/:id`                           | Delete project                                             | Yes  | Yes          |
| `PUT`    | `/api/projects/:id/interview`                 | Save interview responses (batch)                           | Yes  | Yes          |
| `GET`    | `/api/projects/:id/interview`                 | Get interview responses                                    | Yes  | Yes          |
| `POST`   | `/api/projects/:id/figures`                   | Upload figure image (multipart)                            | Yes  | Yes          |
| `DELETE` | `/api/projects/:id/figures/:figure_id`        | Delete a figure                                            | Yes  | Yes          |
| `GET`    | `/api/projects/:id/figures`                   | List figures for project                                   | Yes  | Yes          |
| `POST`   | `/api/projects/:id/generate`                  | Start AI generation (returns SSE stream)                   | Yes  | Yes          |
| `PUT`    | `/api/projects/:id/sections/:type`            | Update section content (manual edit)                       | Yes  | Yes          |
| `POST`   | `/api/projects/:id/sections/:type/regenerate` | Regenerate single section (SSE stream)                     | Yes  | Yes          |
| `POST`   | `/api/projects/:id/export`                    | Generate export (PDF or DOCX)                              | Yes  | Yes          |
| `GET`    | `/api/projects/:id/exports`                   | List exports for project                                   | Yes  | Yes          |
| `GET`    | `/api/exports/:id/download`                   | Get signed download URL                                    | Yes  | Yes          |

- **Auth** = valid JWT in Authorization header required
- **Subscription** = active subscription required (middleware check)
- `/api/me` endpoints do NOT require subscription — needed for the subscribe flow before payment

---

## 14. Environment Variables

### Frontend (.env.local)

```
NEXT_PUBLIC_SUPABASE_URL=https://xxx.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJ...
NEXT_PUBLIC_API_URL=http://localhost:3001  # Rust backend URL
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRICE_ID=price_...
```

### Backend (.env)

```
DATABASE_URL=postgresql://...
SUPABASE_JWT_SECRET=your-jwt-secret
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...  # optional, for future use
SUPABASE_URL=https://xxx.supabase.co
SUPABASE_SERVICE_KEY=eyJ...  # service role key for storage
RUST_LOG=info
PORT=3001
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

- `test_upsert_user_from_jwt` — POST /api/me with valid Supabase JWT creates user row
- `test_upsert_user_idempotent` — POST /api/me called twice doesn't duplicate
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
- `test_delete_project_cascades` — DELETE removes project + interview_responses + sections + figures + exports
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

- `test_export_pdf` — POST /api/projects/:id/export with format=pdf generates file, stores in Supabase Storage, creates exports row
- `test_export_docx` — same for DOCX
- `test_export_download_url` — GET /api/exports/:id/download returns signed URL
- `test_export_requires_all_sections` — export with missing sections returns 400 with list of missing sections

### Frontend Unit Tests (Vitest + React Testing Library)

**Components:**

- `InterviewStep1.test.tsx` — renders form fields, validates required fields, calls onNext with data
- `InterviewStep2.test.tsx` — renders textareas, validates required fields
- `InterviewStep3.test.tsx` — renders textareas, validates required fields
- `InterviewStep4.test.tsx` — renders textareas, optional field can be empty
- `InterviewStep5.test.tsx` — renders file upload, handles multiple images, figure descriptions
- `InterviewReview.test.tsx` — renders all responses, edit buttons navigate to correct step, generate button disabled when fields missing
- `SectionCard.test.tsx` — renders section content, toggles edit mode, shows status badge
- `SectionCard.test.tsx` — edit mode: textarea, save/cancel buttons, auto-save triggers
- `SectionCard.test.tsx` — regenerate button shows confirmation dialog
- `SectionCard.test.tsx` — collapsed state shows first 5 lines + "Show more"
- `ProjectCard.test.tsx` — renders title, status, date, click navigates
- `ExportPage.test.tsx` — renders format buttons, shows progress, lists previous exports
- `SubscribePage.test.tsx` — renders plan details, subscribe button triggers Stripe checkout redirect
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

- `LoginPage.test.tsx` — login form submits to Supabase, redirects to /projects on success, shows error on failure
- `ProjectsPage.test.tsx` — fetches and renders project list, empty state shows "Create your first patent draft" CTA
- `NewProjectPage.test.tsx` — full wizard flow: Step 1 → 2 → 3 → 4 → 5 → 6, saves responses at each step, generate button triggers API call
- `EditorPage.test.tsx` — loads project with sections, renders all section cards, edit saves to API, regenerate streams SSE
- `EditorPage.test.tsx` — generation in progress: shows streaming sections with progress indicator
- `ExportPage.test.tsx` — triggers export, shows progress, handles download URL response
- `SubscribePage.test.tsx` — subscribe flow creates Stripe Checkout session, redirects to Stripe
- `AccountPage.test.tsx` — loads user profile + subscription, manage billing links to Stripe Portal

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

- `interview.spec.ts` — complete full wizard (Steps 1-6) with valid data → all fields saved
- `interview.spec.ts` — navigate back and forth between steps → data persists
- `interview.spec.ts` — refresh browser mid-wizard → resumes at correct step with data intact
- `interview.spec.ts` — skip optional fields (Step 4 alternative embodiments, Step 5 figures) → proceeds normally
- `interview.spec.ts` — try to proceed with empty required fields → validation error shown

**AI generation:**

- `generation.spec.ts` — click "Generate" on Step 6 → sections appear one by one with streaming animation
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
- `subscription.spec.ts` — click "Subscribe" → redirected to Stripe Checkout (verify redirect URL)
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
- Document versioning and diff view
- Razorpay integration for Indian payment preferences
