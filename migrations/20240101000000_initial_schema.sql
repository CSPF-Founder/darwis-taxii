-- Initial TAXII database schema
-- This migration is idempotent for OpenTAXII database compatibility
-- Compatible with PostgreSQL 9.4+

-- ============================================
-- Shared Tables
-- ============================================

CREATE TABLE IF NOT EXISTS accounts (
    id SERIAL PRIMARY KEY,
    username VARCHAR(256) UNIQUE NOT NULL,
    password_hash VARCHAR(256) NOT NULL,
    is_admin BOOLEAN DEFAULT FALSE,
    _permissions TEXT NOT NULL DEFAULT '{}'
);

-- ============================================
-- TAXII 1.x Tables
-- ============================================

CREATE TABLE IF NOT EXISTS services (
    id VARCHAR(150) PRIMARY KEY,
    type VARCHAR(150) NOT NULL,
    _properties TEXT NOT NULL DEFAULT '{}',
    date_created TIMESTAMPTZ DEFAULT NOW(),
    date_updated TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS data_collections (
    id SERIAL PRIMARY KEY,
    name VARCHAR(300) NOT NULL,
    type VARCHAR(150) NOT NULL DEFAULT 'DATA_FEED',
    description TEXT,
    accept_all_content BOOLEAN DEFAULT FALSE,
    bindings TEXT,
    available BOOLEAN DEFAULT TRUE,
    volume INTEGER DEFAULT 0,
    date_created TIMESTAMPTZ DEFAULT NOW()
);

-- OpenTAXII index
DO $$ BEGIN
    CREATE UNIQUE INDEX ix_data_collections_name ON data_collections(name);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS service_to_collection (
    service_id VARCHAR(150) REFERENCES services(id) ON DELETE CASCADE,
    collection_id INTEGER REFERENCES data_collections(id) ON DELETE CASCADE,
    PRIMARY KEY (service_id, collection_id)
);

CREATE TABLE IF NOT EXISTS inbox_messages (
    id SERIAL PRIMARY KEY,
    message_id TEXT NOT NULL,
    result_id TEXT,
    record_count INTEGER,
    partial_count BOOLEAN DEFAULT FALSE,
    subscription_collection_name TEXT,
    subscription_id TEXT,
    exclusive_begin_timestamp_label TIMESTAMPTZ,
    inclusive_end_timestamp_label TIMESTAMPTZ,
    original_message BYTEA NOT NULL,
    content_block_count INTEGER NOT NULL,
    destination_collections TEXT,
    service_id VARCHAR(150) REFERENCES services(id) ON UPDATE CASCADE ON DELETE CASCADE,
    date_created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS content_blocks (
    id SERIAL PRIMARY KEY,
    message TEXT,
    timestamp_label TIMESTAMPTZ DEFAULT NOW(),
    inbox_message_id INTEGER REFERENCES inbox_messages(id) ON UPDATE CASCADE ON DELETE CASCADE,
    content BYTEA NOT NULL,
    binding_id VARCHAR(300),
    binding_subtype VARCHAR(300),
    date_created TIMESTAMPTZ DEFAULT NOW()
);

-- OpenTAXII indexes for content_blocks
DO $$ BEGIN
    CREATE INDEX ix_content_blocks_timestamp_label ON content_blocks(timestamp_label);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_content_blocks_binding_id ON content_blocks(binding_id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_content_blocks_binding_subtype ON content_blocks(binding_subtype);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS collection_to_content_block (
    collection_id INTEGER REFERENCES data_collections(id) ON DELETE CASCADE,
    content_block_id INTEGER REFERENCES content_blocks(id) ON DELETE CASCADE,
    PRIMARY KEY (collection_id, content_block_id)
);

DO $$ BEGIN
    CREATE INDEX ix_collection_to_content_block_content_block_id ON collection_to_content_block(content_block_id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS result_sets (
    id VARCHAR(150) PRIMARY KEY,
    collection_id INTEGER REFERENCES data_collections(id) ON UPDATE CASCADE ON DELETE CASCADE,
    bindings TEXT,
    begin_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    date_created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id VARCHAR(150) PRIMARY KEY,
    collection_id INTEGER REFERENCES data_collections(id) ON UPDATE CASCADE ON DELETE CASCADE,
    params TEXT,
    status VARCHAR(150) NOT NULL DEFAULT 'ACTIVE',
    service_id VARCHAR(150) REFERENCES services(id) ON UPDATE CASCADE ON DELETE CASCADE,
    date_created TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- TAXII 2.x Tables
-- Note: Uses timestamp WITHOUT time zone and json (not jsonb)
-- to match OpenTAXII Python schema exactly
-- ============================================

-- Create ENUM types if they don't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'job_status_enum') THEN
        CREATE TYPE job_status_enum AS ENUM ('pending', 'complete');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'job_detail_status_enum') THEN
        CREATE TYPE job_detail_status_enum AS ENUM ('success', 'failure', 'pending');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS opentaxii_api_root (
    id UUID PRIMARY KEY,
    title VARCHAR(100) NOT NULL,
    description TEXT,
    "default" BOOLEAN NOT NULL DEFAULT FALSE,
    is_public BOOLEAN NOT NULL DEFAULT TRUE
);

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_api_root_title ON opentaxii_api_root(title);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS opentaxii_collection (
    id UUID PRIMARY KEY,
    api_root_id UUID REFERENCES opentaxii_api_root(id) ON DELETE CASCADE,
    title VARCHAR(100) NOT NULL,
    description TEXT,
    alias VARCHAR(100),
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    is_public_write BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (api_root_id, alias)
);

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_collection_title ON opentaxii_collection(title);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

-- Note: Uses timestamp (without timezone) and json (not jsonb) to match OpenTAXII
CREATE TABLE IF NOT EXISTS opentaxii_stixobject (
    pk UUID PRIMARY KEY,
    id VARCHAR(100) NOT NULL,
    collection_id UUID REFERENCES opentaxii_collection(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,
    spec_version VARCHAR(10) NOT NULL DEFAULT '2.1',
    date_added TIMESTAMP NOT NULL DEFAULT NOW(),
    version TIMESTAMP NOT NULL,
    serialized_data JSON NOT NULL,
    UNIQUE (collection_id, id, version)
);

-- OpenTAXII indexes for stixobject
DO $$ BEGIN
    CREATE INDEX ix_opentaxii_stixobject_id ON opentaxii_stixobject(id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_stixobject_type ON opentaxii_stixobject(type);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_stixobject_spec_version ON opentaxii_stixobject(spec_version);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_stixobject_date_added ON opentaxii_stixobject(date_added);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_stixobject_version ON opentaxii_stixobject(version);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_stixobject_collection_id ON opentaxii_stixobject(collection_id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_stixobject_date_added_id ON opentaxii_stixobject(date_added, id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS opentaxii_job (
    id UUID PRIMARY KEY,
    api_root_id UUID REFERENCES opentaxii_api_root(id) ON DELETE CASCADE,
    status job_status_enum NOT NULL DEFAULT 'pending',
    request_timestamp TIMESTAMP,
    completed_timestamp TIMESTAMP,
    total_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    pending_count INTEGER DEFAULT 0
);

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_job_api_root_id_id ON opentaxii_job(api_root_id, id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS opentaxii_job_detail (
    id UUID PRIMARY KEY,
    job_id UUID REFERENCES opentaxii_job(id) ON DELETE CASCADE,
    stix_id VARCHAR(100) NOT NULL,
    version TIMESTAMP NOT NULL,
    message TEXT,
    status job_detail_status_enum NOT NULL DEFAULT 'pending'
);

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_job_detail_job_id ON opentaxii_job_detail(job_id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;

DO $$ BEGIN
    CREATE INDEX ix_opentaxii_job_detail_stix_id ON opentaxii_job_detail(stix_id);
EXCEPTION WHEN duplicate_table THEN NULL;
END $$;
