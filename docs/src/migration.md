# Migration from OpenTAXII

Migrate from [EclecticIQ OpenTAXII](https://github.com/EclecticIQ/OpenTAXII) (Python) to DARWIS TAXII (Rust).

> [!IMPORTANT]
> We recommend testing the migration in a UAT/staging environment first before applying to production. This ensures your specific configuration and data migrate correctly.

## Compatibility

DARWIS TAXII is fully compatible with OpenTAXII:

- **Same database schema** - Point to your existing PostgreSQL database
- **Same password hashes** - Werkzeug (scrypt) hashes work without changes
- **Same data format** - All STIX content is preserved

## Migration Steps

### 1. Stop OpenTAXII

```bash
docker compose down
# or
systemctl stop opentaxii
```

### 2. Backup Database

```bash
pg_dump -h localhost -U user -d opentaxii > opentaxii_backup.sql
```

### 3. Convert Server Configuration

**OpenTAXII (`opentaxii.yml`):**
```yaml
domain: "localhost:9000"
support_basic_auth: true

persistence_api:
  class: opentaxii.persistence.sqldb.SQLDatabaseAPI
  parameters:
    db_connection: postgresql://user:pass@localhost/opentaxii

auth_api:
  class: opentaxii.auth.sqldb.SQLDatabaseAPI
  parameters:
    db_connection: postgresql://user:pass@localhost/opentaxii
    secret: your-secret-key
```

**DARWIS TAXII (`taxii.toml`):**
```toml
domain = "localhost:9000"
support_basic_auth = true

[database]
url = "postgresql://user:pass@localhost/opentaxii"

[auth]
secret = "your-secret-key"
```

### 4. Convert Data Configuration

#### Services

**OpenTAXII:**
```yaml
services:
  - id: inbox
    type: inbox
    address: /services/inbox
    description: Inbox service
```

**DARWIS TAXII:**
```yaml
services:
  - id: inbox
    type: INBOX                      # Uppercase
    properties:                      # Nested under properties
      path: /services/inbox          # 'address' → 'path'
      description: Inbox service
```

| OpenTAXII | DARWIS TAXII |
|-----------|--------------|
| `type: inbox` | `type: INBOX` |
| `type: discovery` | `type: DISCOVERY` |
| `type: poll` | `type: POLL` |
| `type: collection_management` | `type: COLLECTION_MANAGEMENT` |
| `address` | `properties.path` |

#### Collections

**OpenTAXII:**
```yaml
collections:
  - name: my-collection
    available: yes
    accept_all_content: yes
    supported_content:
      - urn:stix.mitre.org:xml:1.1.1
```

**DARWIS TAXII:**
```yaml
collections:
  - name: my-collection
    available: true                  # 'yes' → 'true'
    accept_all_content: true
    supported_content:
      - binding: urn:stix.mitre.org:xml:1.1.1  # Nested under 'binding'
```

#### Accounts

**OpenTAXII:**
```yaml
accounts:
  - username: admin
    password: admin
    is_admin: yes
    permissions:
      my-collection: modify
```

**DARWIS TAXII:**
```yaml
accounts:
  - username: admin
    password: admin
    is_admin: true                   # 'yes' → 'true'
    permissions:
      my-collection: modify          # Same, or use [read, write]
```

### 5. Start DARWIS TAXII

```bash
# Migrations run automatically on startup
./taxii-server

# Or with Docker
docker compose up -d
```

### 6. Verify Migration

```bash
# Check accounts
taxii-cli account list

# Test authentication
curl -X POST http://localhost:9000/management/auth \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "your-password"}'

# Test TAXII 1.x
curl -X POST http://localhost:9000/services/discovery \
  -H "Content-Type: application/xml"

# Test TAXII 2.x
curl http://localhost:9000/taxii2/
```

## Configuration Mapping

| OpenTAXII (YAML) | DARWIS TAXII (TOML) |
|------------------|---------------------|
| `domain` | `domain` |
| `support_basic_auth` | `support_basic_auth` |
| `persistence_api.parameters.db_connection` | `database.url` |
| `auth_api.parameters.secret` | `auth.secret` |
| `auth_api.parameters.token_ttl_secs` | `auth.token_ttl_secs` |

## Environment Variables

| OpenTAXII | DARWIS TAXII |
|-----------|--------------|
| `OPENTAXII_DOMAIN` | `DARWIS_TAXII_DOMAIN` |
| `DATABASE_HOST`, `DATABASE_NAME` | `DARWIS_TAXII_DB_CONNECTION` |
| `AUTH_SECRET` | `DARWIS_TAXII_AUTH_SECRET` |

## Database Compatibility

DARWIS TAXII uses the same schema as OpenTAXII:

**Shared tables:**
- `accounts` - User authentication
- `services` - TAXII 1.x services
- `data_collections` - TAXII 1.x collections
- `content_blocks` - STIX content
- `inbox_messages` - Inbox records
- `subscriptions` - TAXII 1.x subscriptions
- `result_sets` - Poll result sets

**TAXII 2.x tables** (prefixed):
- `opentaxii_api_root`
- `opentaxii_collection`
- `opentaxii_stixobject`
- `opentaxii_job`
- `opentaxii_job_detail`

## Password Compatibility

Werkzeug scrypt hashes are fully supported:

```
scrypt:32768:8:1$salt$hash
```

Existing accounts work without modification.

## Rollback

To rollback to OpenTAXII:

1. Stop DARWIS TAXII
2. Restore backup if needed: `psql -d opentaxii < opentaxii_backup.sql`
3. Start OpenTAXII

The schema is compatible in both directions.

## Differences

| Feature | OpenTAXII | DARWIS TAXII |
|---------|-----------|--------------|
| Language | Python | Rust |
| Config format | YAML | TOML (server), YAML (data) |
| Custom persistence | Plugin API | Not supported |
| Separate auth DB | Supported | Single DB only |

## Troubleshooting

### Authentication fails

Ensure `auth.secret` in `taxii.toml` matches the `secret` from OpenTAXII.

### Database connection errors

Check connection string format:
```
postgresql://username:password@host:port/database
```

### Missing TAXII 2.x data

TAXII 2.x resources must be created via CLI:
```bash
taxii-cli api-root add --title "Default" --default
taxii-cli collection add --api-root-id <uuid> --title "My Collection"
```
