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

Synchronize configuration from YAML file. Manages services, collections, and accounts.

```bash
taxii-cli sync <CONFIG_FILE>
```

**Examples:**
```bash
# Sync configuration
taxii-cli sync data-config.yaml
```

The sync behavior is controlled via YAML options (not CLI flags). See [Sync Configuration](#sync-configuration) below.

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

## Sync Configuration

The `sync` command behavior is controlled entirely via YAML options, making configuration declarative and version-controllable.

### YAML Structure

```yaml
# Entity cleanup behavior (what happens to entities NOT in this file)
prune_services: false            # Delete services not in config (default: false)
collections_not_in_config: ignore # ignore | disable | delete (default: ignore)
prune_accounts: false            # Delete accounts not in config (default: false)

# Entity definitions
services:
  - id: discovery
    type: DISCOVERY
    # ...

collections:
  - name: my-collection
    # ...

accounts:
  - username: admin
    # ...
```

### Cleanup Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `prune_services` | `true`/`false` | `false` | Delete services not in config |
| `collections_not_in_config` | `ignore`/`disable`/`delete` | `ignore` | Action for collections not in config |
| `prune_accounts` | `true`/`false` | `false` | Delete accounts not in config |

### Collection Cleanup Actions

Collections support three actions since they have an "available" flag:

| Value | Behavior |
|-------|----------|
| `ignore` | Leave untouched (default) |
| `disable` | Set `available=false` |
| `delete` | Permanently delete |

> [!CAUTION]
> `delete` permanently removes collections and their content. Use with care.

### Common Patterns

**Additive sync (default):** Only create/update, never delete:
```yaml
# All cleanup options default to safe values
services:
  - id: inbox
    # ...
```

**Full declarative control:** Config is the source of truth:
```yaml
prune_services: true
collections_not_in_config: delete
prune_accounts: true

services:
  # Only these services will exist
collections:
  # Only these collections will exist
accounts:
  # Only these accounts will exist
```

**Accounts-only sync:** Manage accounts without affecting other entities:
```yaml
prune_accounts: true
# prune_services and collections_not_in_config default to safe values

accounts:
  - username: admin
    password: secret
    is_admin: true
```

### Collection Reference Validation

When syncing accounts, all collection references in permissions are validated:

- TAXII 1.x permissions: collection name must exist
- TAXII 2.x permissions: collection UUID must exist

If any referenced collection doesn't exist, the sync fails with an error:

```
Account 'analyst' references non-existent collections:
  - 'invalid-collection' (TAXII 1.x)
  - '00000000-0000-0000-0000-000000000000' (TAXII 2.x)
```

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
