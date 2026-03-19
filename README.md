# Patent Draft Pro

AI-powered patent drafting for the Indian Patent Office (IPO). Guided interview, AI-generated specifications, section editor, PDF/DOCX export.

## Architecture

- **Backend**: Rust (Axum) — API server, AI pipeline, auth, billing webhooks
- **Frontend**: Next.js 16 (App Router) — dark mode, Geist fonts, Tailwind CSS
- **Database**: Postgres (Docker local, Supabase production)
- **Storage**: Local filesystem (dev) / Cloudflare R2 (production)
- **AI**: Provider-agnostic (Claude default, mock for dev)
- **Billing**: Razorpay (subscriptions, UPI + cards)

## Prerequisites

- Rust 1.94+ (`rustup update stable`)
- Node.js 20+
- Docker (for local Postgres)
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres`)

## Quick Start

```bash
# 1. Copy environment variables
cp .env.example .env

# 2. Start Postgres
make db-up

# 3. Run migrations
make migrate

# 4. Create test user (with active subscription)
make seed-user

# 5. Start backend (port 5012)
make dev-backend

# 6. In another terminal — start frontend (port 3000)
make dev-frontend

# 7. Open http://localhost:3000/login
#    Email: test@example.com
#    Password: testpass123
```

## Development Commands

| Command | Description |
|---------|-------------|
| `make db-up` | Start Postgres via Docker Compose |
| `make db-down` | Stop Postgres |
| `make migrate` | Run database migrations |
| `make seed-user` | Create test user with active subscription |
| `make dev-backend` | Start Rust API server (port 5012) |
| `make dev-frontend` | Start Next.js dev server (port 3000) |
| `make test-backend` | Run Rust unit + integration tests |
| `make test-frontend` | Run Vitest frontend tests |
| `make test` | Run all tests |
| `npm run test:e2e` | Run Playwright E2E tests |

## Project Structure

```
├── apps/web/              Next.js 16 frontend
│   ├── app/               Pages (login, projects, editor, export, account)
│   ├── components/        UI components (interview, editor, layout)
│   ├── hooks/             React hooks (auth, SSE stream, auto-save, wizard)
│   └── lib/               API client, utilities
├── backend/               Rust Cargo workspace
│   ├── api/               Axum HTTP server + routes + middleware
│   ├── ai/                LLM provider trait + pipeline
│   ├── export/            PDF (typst) + DOCX generation
│   ├── storage/           Local filesystem + Cloudflare R2
│   └── shared/            DB models, config, connection pool
├── migrations/            SQL migrations (11 files)
├── e2e/                   Playwright E2E tests
└── .github/workflows/     CI (Rust + Frontend)
```

## API Endpoints

All protected routes require JWT auth (httpOnly cookie) + active subscription.

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/auth/login` | Login (sets cookies) |
| POST | `/api/auth/refresh` | Refresh JWT |
| POST | `/api/auth/logout` | Logout (clears cookies) |
| GET | `/api/me` | User profile + subscription status |
| GET/POST | `/api/projects` | List / create projects |
| GET/PATCH/DELETE | `/api/projects/:id` | Get / update / soft-delete |
| PUT/GET | `/api/projects/:id/applicant` | Upsert / get applicant details |
| PUT/GET | `/api/projects/:id/interview` | Save / get interview responses |
| POST/GET/DELETE | `/api/projects/:id/figures` | Upload / list / delete figures |
| POST | `/api/projects/:id/generate` | Start AI generation (SSE stream) |
| PUT | `/api/projects/:id/sections/:type` | Update section content |
| GET | `/api/projects/:id/sections/:type/versions` | Version history |
| POST | `/api/projects/:id/export` | Generate PDF/DOCX |
| GET | `/api/exports/:id/download` | Download URL |

## Test User

```
Email: test@example.com
Password: testpass123
```

Created by `make seed-user` with an active subscription (expires 1 year from creation).

## Deployment

- **Backend**: Fly.io (Mumbai region) — `fly deploy`
- **Frontend**: Vercel — auto-deploys from GitHub
- **Database**: Supabase Postgres (managed)
- **Storage**: Cloudflare R2
