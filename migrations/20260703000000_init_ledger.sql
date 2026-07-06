-- Create extension and custom enum type for account types
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TYPE account_type AS ENUM ('asset', 'liability', 'equity', 'revenue', 'expense');

-- Chart of accounts table
CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type account_type NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Core transaction headers (idempotency target)
CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    idempotency_key VARCHAR(255) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Double-entry transaction lines
CREATE TABLE entries (
    id UUID PRIMARY KEY,
    transaction_id UUID REFERENCES transactions(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id),
    amount BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Immutable central database auditing store
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(100) NOT NULL,
    action_type VARCHAR(10) NOT NULL,
    record_id UUID NOT NULL,
    old_data JSONB,
    new_data JSONB,
    db_user VARCHAR(100) NOT NULL,
    app_user VARCHAR(250),
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance optimization and tuning
CREATE INDEX idx_entries_tx_id ON entries(transaction_id);
CREATE INDEX idx_entries_account_id ON entries(account_id);

-- PL/pgSQL trigger function to log mutation rows clearly and immutably
CREATE OR REPLACE FUNCTION process_row_audit()
RETURNS TRIGGER AS $$
DECLARE
    current_app_user TEXT;
BEGIN
    BEGIN
        current_app_user := current_setting('app.current_user', true);
    EXCEPTION WHEN OTHERS THEN
        current_app_user := 'system_worker';
    END;

    IF (TG_OP = 'DELETE') THEN
        INSERT INTO audit_logs (table_name, action_type, record_id, old_data, new_data, db_user, app_user)
        VALUES (TG_TABLE_NAME, TG_OP, OLD.id, to_jsonb(OLD), NULL, current_user, current_app_user);
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit_logs (table_name, action_type, record_id, old_data, new_data, db_user, app_user)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, to_jsonb(OLD), to_jsonb(NEW), current_user, current_app_user);
        RETURN NEW;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO audit_logs (table_name, action_type, record_id, old_data, new_data, db_user, app_user)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, NULL, to_jsonb(NEW), current_user, current_app_user);
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Bind the trigger function to the relevant tables for auditing
CREATE TRIGGER audit_transactions_trigger
BEFORE INSERT OR UPDATE OR DELETE ON transactions
FOR EACH ROW EXECUTE FUNCTION process_row_audit();

CREATE TRIGGER audit_entries_trigger
BEFORE INSERT OR UPDATE OR DELETE ON entries
FOR EACH ROW EXECUTE FUNCTION process_row_audit();

-- Seed generic accounts for the chart of accounts
INSERT INTO accounts (id, name, type) VALUES
    ('8f14e45f-ea3b-4c1b-9d2e-1c2b5e5f6a1a', 'Cash', 'asset'),
    ('9c8e7d6f-4b3a-4d2c-8e1f-2b3c4d5e6f7a', 'Accounts Receivable', 'asset');

    
