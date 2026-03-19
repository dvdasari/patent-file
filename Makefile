.PHONY: dev dev-backend dev-frontend db-up db-down migrate seed-user test test-backend test-frontend test-e2e setup-tools clean

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
