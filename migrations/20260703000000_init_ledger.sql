-- Create custom enum type for account types
CREATE TYPE account_type AS ENUM ('asset', 'liability', 'equity', 'revenue', 'expense');

-- Chart of accounts table
CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type account_type NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
);

-- Core transaction headers (Idenpotency target)
CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    idempotency_key VARCHAR(255) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
);

-- Double-entry transaction lines
CREATE TABLE entries (
    id UUID PRIMARY KEY,
    transaction_id UUID REFERENCES transactions(id) ON DELETE CASCADE,
    account_id UUID Not NULL REFERENCES accounts(id),
    amount BigInt NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

--  Immuttable central database auditing store
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

-- Indexes for performance optimization and tunning
CREATE INDEX idx_entries_tx_id ON entries(transaction_id);
CREATE INDEX idx_entries_account_id ON entries(account_id);

-- PL/PgSQL Trigger function to log mutation rows clearly and immutably
CREATE OR REPLACE FUNCTION FUNCTION process_row_audit() 
RETURNS TRIGGER AS $$
DECLARE
    current_user_name TEXT;
BEGIN
    -- Get the current database user
    BEGIN
        current_app_user := current_setting('app.current_user', true);
    EXEPTION WHEN OTHERS THEN
        current_app_user := 'system_worker';
    END;    

    -- Handle different operations (INSERT, UPDATE, DELETE)
    IF (TG_OP = 'DELETE') THEN 
        INSERT INTO audit_logs (table_name, action_type, record_id, old_state, new_state, db_user, app_user)
        VALUES (TG_TABLE_NAME, TG_OP, OLD.id, to_jsonb(OLD), NULL,  current_user, current_app_user);
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit_logs (table_name, action_type, record_id, old_state, new_state, db_user, app_user)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, to_jsonb(OLD), to_jsonb(NEW), current_user, current_app_user);
        RETURN NEW;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO audit_logs (table_name, action_type, record_id, old_state, new_state, db_user, app_user)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, NULL, to_jsonb(NEW), current_user, current_app_user);
        RETURN NEW;
    END IF;
    RETURN NULL; -- result is ignored since this is an AFTER trigger
END;
$$ LANGUAGE plpgsql;    

-- Bind the trigger function to the relevant tables for auditing
CREATE TRIGGER audit_transactions_trigger BEFORE INSERT OR UPDATE OR DELETE ON transactions
FOR EACH ROW EXECUTE FUNCTION process_row_audit();
CREATE TRIGGER audit_entries_trigger BEFORE INSERT OR UPDATE OR DELETE ON entries
FOR EACH ROW EXECUTE FUNCTION process_row_audit();  


-- SEED genric accounts for the chart of accounts
INSERT INTO accounts (id, name, type) VALUES
    ('8f14e45f-ea3b-4c1b-9d2e-1c2b5e5f6a1a', 'Cash', 'asset'),
    ('9c8e7d6f-4b3a-4d2c-8e1f-2b3c4d5e6f7g', 'Accounts Receivable', 'asset');

    
