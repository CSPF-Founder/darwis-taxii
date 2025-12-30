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

**This is critical:** The permission key format differs between TAXII versions.

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

Both TAXII 1.x and 2.x style values are supported:

### TAXII 1.x Style (String)

```yaml
permissions:
  my-collection: read     # Read-only
  other-collection: modify # Read + write
```

### TAXII 2.x Style (List)

```yaml
permissions:
  86c1741e-...: [read]         # Read-only
  24574d4d-...: [read, write]  # Full access
```

### Equivalence

| TAXII 1.x | TAXII 2.x | Access Level |
|-----------|-----------|--------------|
| `read` | `[read]` | Read-only |
| `modify` | `[read, write]` | Read + Write |

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
- Does NOT delete accounts not in config

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
