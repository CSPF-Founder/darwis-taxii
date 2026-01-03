# Permissions

Account permissions control access to TAXII collections.

## Permission Model

Permissions are defined per-user, per-collection:

```yaml
accounts:
  - username: analyst
    password: secret
    is_admin: false
    permissions:
      collection-key: permission-value
```

## Admin Accounts

Accounts with `is_admin: true` have full access to all collections:

```yaml
accounts:
  - username: admin
    password: changeme
    is_admin: true
    # No permissions needed - admin has full access
```

## Collection Keys

> [!IMPORTANT]
> TAXII 1.x and TAXII 2.x collections are **completely separate**. They are stored in different database tables and cannot be shared between protocols.

**The permission key format differs between TAXII versions:**

### TAXII 1.x: Use Collection Name

TAXII 1.x collections are identified by their `name`:

```yaml
# data-config.yaml
collections:
  - name: my-collection    # ← This is the identifier
    type: DATA_FEED

accounts:
  - username: analyst
    permissions:
      my-collection: read  # ← Use the name
```

### TAXII 2.x: Use Collection UUID

TAXII 2.x collections are identified by their UUID (`id`):

```yaml
accounts:
  - username: analyst
    permissions:
      # Use the UUID, NOT the title
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read, write]
```

**Finding collection UUIDs:**
```bash
taxii-cli collection list --api-root-id <api-root-uuid>
```

### Why Different?

| Protocol | Identifier | Why |
|----------|------------|-----|
| TAXII 1.x | `name` | TAXII 1.x has no UUID concept |
| TAXII 2.x | `id` (UUID) | Per spec, UUID is THE identifier |

**Common mistake:** Using the TAXII 2.x collection `title` instead of UUID:

```yaml
# WRONG - title is not the identifier
permissions:
  "My Collection": [read, write]

# CORRECT - use UUID
permissions:
  86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read, write]
```

## Permission Values

**Critical:** The value format determines which protocol the permission applies to.

### TAXII 1.x: String Values

TAXII 1.x permissions use **string** values:

```yaml
permissions:
  my-collection: read     # Read-only
  other-collection: modify # Read + write
```

| Value | Access |
|-------|--------|
| `read` | Read-only |
| `modify` | Read + write |

### TAXII 2.x: List Values

TAXII 2.x permissions use **list** values:

```yaml
permissions:
  86c1741e-...: [read]         # Read-only
  24574d4d-...: [write]        # Write-only (submit without reading)
  f8c3e7a2-...: [read, write]  # Full access
```

| Value | Access |
|-------|--------|
| `[read]` | Read-only |
| `[write]` | Write-only |
| `[read, write]` | Full access |

### Format Summary

| Protocol | Key | Value Format | Example |
|----------|-----|--------------|---------|
| TAXII 1.x | Collection name | String | `my-collection: modify` |
| TAXII 2.x | Collection UUID | List | `86c1741e-...: [read, write]` |

> [!CAUTION]
> Using the wrong format will cause validation errors. A UUID with a string value is treated as TAXII 1.x and will fail collection validation.

## Mixed Permissions

You can mix TAXII 1.x and 2.x permissions on the same user:

```yaml
accounts:
  - username: analyst
    permissions:
      # TAXII 1.x collections (by name)
      threat-intel: read
      incident-data: modify

      # TAXII 2.x collections (by UUID)
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read, write]
      24574d4d-d29a-4b53-80c0-be454dfac6d5: [read]
```

## Complete Example

```yaml
accounts:
  # Admin - full access
  - username: admin
    password: admin-secret
    is_admin: true

  # Analyst - mixed TAXII 1.x and 2.x
  - username: analyst
    password: analyst-secret
    is_admin: false
    permissions:
      # TAXII 1.x collections
      legacy-feed: modify

      # TAXII 2.x collections
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read, write]

  # Read-only user
  - username: reader
    password: reader-secret
    is_admin: false
    permissions:
      legacy-feed: read
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read]

  # No permissions (can authenticate but access nothing)
  - username: pending
    password: pending-secret
    is_admin: false
    # No permissions defined
```

## Applying Permissions

Permissions are synced from your configuration file:

```bash
taxii-cli sync data-config.yaml
```

The sync command:
- Creates new accounts
- Updates existing account permissions
- Validates all collection references exist (fails if any are missing)

### Account Cleanup

By default, accounts not in the config file are left untouched. To delete them:

```yaml
prune_accounts: true

accounts:
  - username: admin
    # Only accounts listed here will remain
```

## Collection Validation

Before any changes are made, the sync command validates that all collections referenced in permissions actually exist:

- **TAXII 1.x**: Collection name must exist in `data_collections` table
- **TAXII 2.x**: Collection UUID must exist in `opentaxii_collection` table

If validation fails, no changes are made and an error is shown:

```
Account 'analyst' references non-existent collections:
  - 'unknown-collection' (TAXII 1.x)
  - '00000000-0000-0000-0000-000000000000' (TAXII 2.x)
```

This prevents permissions from referencing collections that don't exist, which could cause confusing behavior at runtime.

## Checking Permissions

### Via CLI

```bash
taxii-cli account list
```

### Via API

The collection response includes permission info:

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/collections/
```

Response shows `can_read` and `can_write` per collection:
```json
{
  "collections": [
    {
      "id": "86c1741e-...",
      "title": "IOC Feed",
      "can_read": true,
      "can_write": false
    }
  ]
}
```

## Public Collections

TAXII 2.x collections can be made public (no auth required):

```bash
# Public read
taxii-cli collection add --api-root-id <id> --title "Public" --public

# Public read + write (use carefully!)
taxii-cli collection add --api-root-id <id> --title "Open" --public --public-write
```

Public collections bypass permission checks for the specified operations.

## Troubleshooting

### "User does not have read access"

1. Check the collection key format:
   - TAXII 1.x: Use collection `name`
   - TAXII 2.x: Use collection `id` (UUID)

2. Verify the UUID is correct:
   ```bash
   taxii-cli collection list --api-root-id <api-root-id>
   ```

3. Re-sync configuration:
   ```bash
   taxii-cli sync data-config.yaml
   ```

### Permissions not taking effect

1. Ensure you ran `taxii-cli sync` after changing config
2. Check the account exists: `taxii-cli account list`
3. Verify the collection UUID matches exactly
