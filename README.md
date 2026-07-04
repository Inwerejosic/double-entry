# Double Entry

Double Entry is a Rust-based ledger service for building and exploring double-entry accounting workflows. The project is structured as a web service with PostgreSQL persistence, SQLx-based data access, and an Actix Web HTTP layer. It is intended to serve as a foundation for managing accounts, posting transactions, and maintaining an auditable ledger.

## Overview

Double-entry accounting is a bookkeeping method where every financial transaction affects at least two accounts so that the accounting equation remains balanced. This project aims to model that idea in a small, extensible Rust application.

At the moment, the repository contains the initial project scaffold and a SQL migration for the ledger schema. The core web handlers and data models are still being implemented, so this README documents the intended architecture and the current state clearly.

## Project goals

- Model a basic chart of accounts
- Support posting double-entry transactions
- Enforce balance through ledger entries
- Store immutable audit information for changes to core tables
- Provide a clean Rust project structure for further API development

## Current status

The repository is in an early development stage.

What is already present:
- A Rust crate named double-entry
- Actix Web and SQLx dependencies
- A PostgreSQL migration for accounts, transactions, entries, and audit logs
- A basic project layout for DAO, DTO, and handler modules

What is still pending or incomplete:
- The main application entrypoint is still a placeholder
- HTTP handlers for transaction processing are not yet implemented
- DAO and DTO modules are currently empty scaffolding
- End-to-end API behavior and tests have not yet been built

## Tech stack

- Rust 2024 edition
- Actix Web for HTTP handling
- SQLx for database access
- PostgreSQL for persistence
- serde and validator for request/response modeling and validation
- tracing and tracing-subscriber for structured logging
- UUID and chrono for identifiers and timestamps

## Repository layout

- Cargo.toml: crate manifest and dependencies
- src/main.rs: application entrypoint
- src/dao/: database access and persistence logic
- src/dto/: request and response DTOs
- src/handlers/: HTTP route handlers
- migrations/: SQL migrations for PostgreSQL

## Database design

The initial migration defines the core ledger schema:

- accounts: stores the chart of accounts
- transactions: stores transaction headers and an idempotency key
- entries: stores ledger lines for each transaction
- audit_logs: stores immutable records of changes to the ledger tables

The schema also includes:
- A custom account_type enum with values for asset, liability, equity, revenue, and expense
- Indexes on transaction and account lookups
- PostgreSQL triggers intended to log mutations into the audit trail

## Prerequisites

Before running the project locally, make sure you have:

- Rust and Cargo installed
- PostgreSQL installed and running
- A database created for the application

## Local setup

1. Clone the repository

   ```bash
   git clone <repository-url>
   cd double-entry
   ```

2. Create a PostgreSQL database

   ```sql
   CREATE DATABASE double_entry;
   ```

3. Configure the database connection string

   Set an environment variable such as:

   ```bash
   export DATABASE_URL=postgres://username:password@localhost:5432/double_entry
   ```

4. Run the migration

   If you are using SQLx offline metadata or a migration runner, apply the SQL migration from the migrations directory.

5. Build the project

   ```bash
   cargo build
   ```

6. Run the application

   ```bash
   cargo run
   ```

## Expected application flow

The planned application flow is:

1. Create accounts in the chart of accounts
2. Post a transaction with a debit and credit side
3. Store the transaction and its related entries in the database
4. Maintain balance and auditability for every change

A typical transaction will likely involve:
- one or more debit entries
- one or more credit entries
- a total debit amount equal to the total credit amount

## API design notes

The project structure suggests an API-first design with the following areas of responsibility:

- DTO modules for input/output validation
- Handler modules for request processing
- DAO modules for persistence operations

The expected future endpoints may include:
- creating accounts
- creating transactions
- listing accounts
- listing transactions or entries
- retrieving account balances or ledger history

These endpoints are not yet implemented in the current scaffold.

## Development workflow

When working on the project:

- Keep the domain model centered on ledger correctness
- Preserve idempotency for transaction creation
- Keep audit logging immutable and append-only in spirit
- Use validation on incoming request payloads
- Keep database changes in migrations so the schema stays reproducible

## Testing

Tests are not yet implemented. As the project matures, the following areas should be covered:

- transaction balance validation
- idempotency handling
- database write and read behavior
- API endpoint behavior
- audit log correctness

## Contributing

Contributions are welcome as the project evolves. A good starting point is to implement the transaction flow, define the HTTP API, and add tests around ledger correctness.

## License

No explicit license has been defined yet. If you intend to publish or share this project more broadly, consider adding one.



