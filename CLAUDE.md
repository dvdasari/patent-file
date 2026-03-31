# Patent File — Project Conventions

AI-powered patent drafting for the Indian Patent Office. Full-stack: Rust (Axum) backend + Next.js 16 frontend.

## Tech Stack

| Layer | Choice | Notes |
|---|---|---|
| Backend | Rust (Axum) | Workspace: `backend/` |
| Frontend | Next.js 16 App Router | `apps/web/` |
| Database | PostgreSQL 16 | Migrations in `migrations/` |
| AI | Claude (Anthropic) | Provider-agnostic trait; mock for dev |
| Billing | Razorpay | Webhooks + subscription gating |
| Storage | Local (dev) / Cloudflare R2 (prod) | |
| Hosting | Fly.io (backend) + Vercel (frontend) | |

## Repository Layout

```
backend/           Rust Cargo workspace
  api/             Axum HTTP server, routes, middleware
  ai/              LLM provider trait + Anthropic/mock implementations
  export/          PDF (typst) + DOCX generation
  storage/         Local FS + Cloudflare R2 client
  shared/          DB models, config, connection pool
apps/web/          Next.js 16 frontend
migrations/        SQL migrations (numbered, sequential)
e2e/               Playwright end-to-end tests
.github/workflows/ CI: rust.yml + frontend.yml
```

## Development Workflow

```bash
cp .env.example .env     # fill in secrets
make db-up               # start Postgres via Docker
make migrate             # run all SQL migrations
make seed-user           # create test@example.com / testpass123 (active subscription)
make dev-backend         # Rust API on :5012
make dev-frontend        # Next.js on :3000
```

## Testing

```bash
make test-backend        # cargo test --workspace (integration tests use testcontainers)
make test-frontend       # vitest run
make test-e2e            # Playwright (requires running app)
```

Backend integration tests spin up a real Postgres via `testcontainers` — no mocking the DB.

## Adding Database Migrations

1. Create `migrations/NNN_description.sql` (next sequential number).
2. Run `make migrate` locally to apply.
3. CI runs `sqlx migrate run` before `cargo test`.

Never modify an existing migration that has been merged to `main`.

## API Conventions

- All routes under `/api/`
- Auth via httpOnly JWT cookie (set on login/refresh)
- Active subscription required for most protected routes
- Errors: `{ "error": "message" }` with appropriate HTTP status

## Environment Variables

See `.env.example` for all required variables. Key ones:

- `DATABASE_URL` — Postgres connection string
- `JWT_SECRET` — min 64 chars
- `AI_PROVIDER` — `"mock"` locally, `"anthropic"` in prod
- `ANTHROPIC_API_KEY` — only needed when `AI_PROVIDER=anthropic`
- `STORAGE_BACKEND` — `"local"` dev, `"r2"` prod

## Code Style

- Rust: `cargo fmt` + `cargo clippy --all-targets` before committing
- TypeScript: `eslint` (config in `apps/web/eslint.config.mjs`)
- No dead code, no `#[allow(unused)]` without a comment explaining why

## CI

Both workflows trigger on push and pull_request:

- `rust.yml` — runs migrations + `cargo test --workspace` against a real Postgres service container
- `frontend.yml` — runs `vitest` + `next build`

PRs must pass both checks before merging.
