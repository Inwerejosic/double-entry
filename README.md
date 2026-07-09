# Double Entry

Double Entry is a Rust-based ledger service for building and exploring double-entry accounting workflows. The project is a web service with PostgreSQL persistence, SQLx-based data access, and an Actix Web HTTP layer. It provides endpoints for health checks and posting balanced double-entry transactions, and includes Docker and CI support for building and scanning container images.

## Overview

Double-entry accounting is a bookkeeping method where every financial transaction affects at least two accounts so that the accounting equation remains balanced. This project models that idea in a compact Rust service with validation, persistence, and audit logging.

## Current status

What is implemented:
- Working Actix Web server with endpoints: `/`, `/health`, `/uptime`, and `POST /v1/transactions`.
- `POST /v1/transactions` accepts validated, balanced double-entry payloads and writes to PostgreSQL using SQLx.
 - `GET /v1/accounts/{account_id}/balance` to fetch an account's current balance (sums `entries.amount`).
- SQL migration in `migrations/20260703000000_init_ledger.sql` creating `accounts`, `transactions`, `entries`, and `audit_logs`, plus audit trigger logic.
- Dockerfile and `docker-compose.yaml` to run the app + Postgres stack locally.
- GitHub Actions workflow `.github/workflows/build-and-scan.yml` that builds the image and runs a Trivy scan before pushing to Docker Hub.

What remains / suggestions:
- Add unit and integration tests for transaction flows and audit triggers.
- Harden CI policy for allowed vulnerability severities and add automated rollbacks if needed.

## Tech stack

- Rust 2024 edition
- Actix Web for HTTP handling
- SQLx for database access
- PostgreSQL for persistence
- serde and validator for request/response modeling and validation
- tracing and tracing-subscriber for structured logging
- UUID and chrono for identifiers and timestamps

## Repository layout

- `Cargo.toml`: crate manifest and dependencies
- `src/main.rs`: application entrypoint
- `src/dao/`: database access and persistence logic
- `src/dto/`: request and response DTOs
- `src/handlers/`: HTTP route handlers
- `migrations/`: SQL migrations for PostgreSQL

## Running the service

Option A — Run with Docker Compose (recommended)

1. Start the stack

```bash
docker compose up -d --build
```

2. Confirm services are healthy

```bash
docker compose ps
docker compose logs --tail 50 app
```

Note: the `docker-compose.yaml` defines the DB service as `postgres` and the app service as `app`.

3. Stop the stack

```bash
docker compose down
```

Option B — Run locally (requires Rust + Postgres)

1. Create the Postgres database and apply the SQL in `migrations/20260703000000_init_ledger.sql`.
2. Set `DATABASE_URL` environment variable, build and run:

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/ledger_db
cargo build --release
cargo run --release
```

## Important binding note

The server binds to `0.0.0.0:8080` so that Docker host requests can reach the container. If you run the binary directly and prefer localhost-only, adjust the bind address in `src/main.rs`.

## Endpoints & sample requests

- `GET /` — root health (same payload as `/health`).
- `GET /health` — returns JSON with service status and uptime.
- `GET /uptime` — same as `/health`.
- `POST /v1/transactions` — post a balanced transaction.

- `GET /v1/accounts/{account_id}/balance` — returns JSON with the account's balance (sum of `entries.amount`).

### Health check (curl)

```bash
curl -v http://127.0.0.1:8080/health
```

Expected JSON response example:

```json
{"status":"ok","uptime_seconds":10}
```

### Transaction example (balanced)

Request

```bash
curl -v -X POST http://127.0.0.1:8080/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key":"tx-1234",
    "description":"Test balanced ledger transaction",
    "entries":[
      {"account_id":"8f14e45f-ea3b-4c1b-9d2e-1c2b5e5f6a1a","amount":100},
      {"account_id":"9c8e7d6f-4b3a-4d2c-8e1f-2b3c4d5e6f7a","amount":-100}
    ]
  }'
```

Successful response (HTTP 201):

```json
{
  "status": "success",
  "transaction_id": "<uuid>"
}
```

Validation errors will return `400` with a JSON payload describing the validation failure. Idempotency conflicts return `409`.

### Account balance example

Request

```bash
curl -v http://127.0.0.1:8080/v1/accounts/8f14e45f-ea3b-4c1b-9d2e-1c2b5e5f6a1a/balance
```

Expected JSON response example (200):

```json
{
  "account_id":"8f14e45f-ea3b-4c1b-9d2e-1c2b5e5f6a1a",
  "balance": 10000
}
```

If balances appear empty, seed a balanced transaction using `POST /v1/transactions` (see example above) or insert rows directly into `entries`/`transactions`. The migration seeds two accounts by default (`Cash` and `Accounts Receivable`).

## CI / Image scanning

This repository contains a GitHub Actions workflow at `.github/workflows/build-and-scan.yml` that:
- builds the Docker image
- runs a Trivy vulnerability scan
- pushes the image to Docker Hub (requires `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` secrets)

Adjust the workflow or CVE severity levels as needed for your security policy.

## Contributing

Contributions are welcome. Good next steps are:
- implement tests for transaction flows and audit triggers
- add more API coverage (accounts listing, balances)
- improve CI policies and add automated integrations

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.
