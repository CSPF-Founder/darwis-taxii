# CLI Reference

The `taxii-cli` command-line tool manages DARWIS TAXII.

## Global Options

```bash
taxii-cli [OPTIONS] <COMMAND>
```

| Option | Description |
|--------|-------------|
| `--config <PATH>` | Path to taxii.toml |
| `--database-url <URL>` | Database connection (overrides config) |
| `-h, --help` | Show help |
| `-V, --version` | Show version |

Database URL can also be set via `DATABASE_URL` environment variable.

## Commands

### sync

Synchronize TAXII 1.x configuration from YAML file.

```bash
taxii-cli sync <CONFIG_FILE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--force-delete` | Delete collections not in config (default: disable) |

**Examples:**
```bash
# Sync configuration
taxii-cli sync data-config.yaml

# Force delete removed collections
taxii-cli sync data-config.yaml --force-delete
```

### api-root

Manage TAXII 2.x API roots.

#### api-root list

List all API roots.

```bash
taxii-cli api-root list
```

#### api-root add

Create a new API root.

```bash
taxii-cli api-root add [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--title <TITLE>` | API root title (required) |
| `--description <DESC>` | Optional description |
| `--default` | Make this the default API root |

**Examples:**
```bash
taxii-cli api-root add --title "Threat Intel" --default
taxii-cli api-root add --title "Partner Sharing" --description "Shared with partners"
```

### collection

Manage TAXII 2.x collections.

#### collection list

List collections in an API root.

```bash
taxii-cli collection list --api-root-id <UUID>
```

#### collection add

Create a new collection.

```bash
taxii-cli collection add [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--api-root-id <UUID>` | API root UUID (required) |
| `--title <TITLE>` | Collection title (required) |
| `--description <DESC>` | Optional description |
| `--alias <ALIAS>` | URL-friendly alias |
| `--public` | Allow unauthenticated read |
| `--public-write` | Allow unauthenticated write |

**Examples:**
```bash
taxii-cli collection add \
  --api-root-id a1b2c3d4-... \
  --title "IOC Feed" \
  --alias iocs

taxii-cli collection add \
  --api-root-id a1b2c3d4-... \
  --title "Public Intel" \
  --public
```

### account

Manage user accounts.

> [!TIP]
> Accounts are created via the `sync` command with a YAML configuration file. See the examples section below.

#### account list

List all accounts.

```bash
taxii-cli account list
```

#### account delete

Delete an account.

```bash
taxii-cli account delete --username <NAME>
```

### content

Manage content blocks (TAXII 1.x).

#### content delete

Delete content blocks from collections.

```bash
taxii-cli content delete [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-c, --collection <NAME>` | Collection name(s) (required, repeatable) |
| `--begin <TIMESTAMP>` | Start of time window (ISO8601) |
| `--end <TIMESTAMP>` | End of time window (optional) |
| `-m, --with-messages` | Also delete inbox messages |

**Examples:**
```bash
# Delete content from January 2024
taxii-cli content delete \
  --collection my-collection \
  --begin 2024-01-01T00:00:00Z \
  --end 2024-02-01T00:00:00Z

# Delete from multiple collections with messages
taxii-cli content delete \
  --collection coll-a \
  --collection coll-b \
  --begin 2024-01-01T00:00:00Z \
  --with-messages
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `DARWIS_TAXII_CONFIG` | Path to taxii.toml |
| `DARWIS_TAXII_AUTH_SECRET` | JWT signing secret |

## Examples

### Complete Setup Workflow

```bash
# 1. Create API root
taxii-cli api-root add --title "Intel Hub" --default

# 2. List to get the UUID
taxii-cli api-root list
# ID: a1b2c3d4-e5f6-7890-abcd-ef1234567890

# 3. Create collections
taxii-cli collection add \
  --api-root-id a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  --title "Indicators" \
  --alias indicators

taxii-cli collection add \
  --api-root-id a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  --title "Malware" \
  --alias malware

# 4. List collections to get UUIDs
taxii-cli collection list \
  --api-root-id a1b2c3d4-e5f6-7890-abcd-ef1234567890

# 5. Create accounts with permissions
cat > data-config.yaml << EOF
accounts:
  - username: analyst
    password: analyst123
    permissions:
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read, write]
      24574d4d-d29a-4b53-80c0-be454dfac6d5: [read]
EOF

taxii-cli sync data-config.yaml
```

### TAXII 1.x Setup

```bash
# Copy template and edit
cp examples/data-config/full.yaml data-config.yaml
# Edit data-config.yaml with your services, collections, accounts

# Apply configuration
taxii-cli sync data-config.yaml
```

### Cleanup Old Data

```bash
# Delete content older than 90 days
taxii-cli content delete \
  --collection threat-intel \
  --begin 1970-01-01T00:00:00Z \
  --end $(date -d '90 days ago' --iso-8601=seconds)Z
```
