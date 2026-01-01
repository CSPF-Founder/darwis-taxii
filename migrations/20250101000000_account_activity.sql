-- Account activity tracking for credential usage monitoring
-- This migration is backward compatible - does not modify existing tables
-- Compatible with PostgreSQL 9.4+

-- ============================================
-- Account Activity Table
-- ============================================

CREATE TABLE IF NOT EXISTS account_activity (
    id BIGSERIAL PRIMARY KEY,
    account_id INT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    event_type VARCHAR(32) NOT NULL,  -- 'login_success', 'login_failed'
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying by account (most common query)
DO $$ BEGIN
    CREATE INDEX ix_account_activity_account_id ON account_activity(account_id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

-- Index for time-based queries and cleanup (30-day retention)
DO $$ BEGIN
    CREATE INDEX ix_account_activity_created_at ON account_activity(created_at);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

-- Composite index for last login queries
DO $$ BEGIN
    CREATE INDEX ix_account_activity_account_event ON account_activity(account_id, event_type, created_at DESC);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;
