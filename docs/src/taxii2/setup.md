# TAXII 2.x Setup & Configuration

TAXII 2.x is configured via CLI commands, not YAML files.

## Create API Root

An API root is required before creating collections:

```bash
# Create a default API root
taxii-cli api-root add --title "Threat Intelligence" --default

# Create additional API roots
taxii-cli api-root add --title "Internal Intel"
taxii-cli api-root add --title "Partner Sharing"
```

**Options:**

| Option | Description |
|--------|-------------|
| `--title` | Human-readable name (required) |
| `--description` | Optional description |
| `--default` | Make this the default API root |

## List API Roots

```bash
taxii-cli api-root list
```

Output:
```
ID                                    Title                 Default
────────────────────────────────────────────────────────────────────
a1b2c3d4-e5f6-7890-abcd-ef1234567890  Threat Intelligence   Yes
b2c3d4e5-f6a7-8901-bcde-f12345678901  Internal Intel        No
```

## Create Collections

Collections must belong to an API root:

```bash
# Create a collection
taxii-cli collection add \
  --api-root-id a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  --title "Malware Indicators"

# Create with alias (for friendly URLs)
taxii-cli collection add \
  --api-root-id a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  --title "IP Blocklist" \
  --alias blocklist

# Create public collection (no auth required for read)
taxii-cli collection add \
  --api-root-id a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  --title "Public Feed" \
  --public
```

**Options:**

| Option | Description |
|--------|-------------|
| `--api-root-id` | API root UUID (required) |
| `--title` | Collection title (required) |
| `--description` | Optional description |
| `--alias` | URL-friendly alias (unique within API root) |
| `--public` | Allow unauthenticated read access |
| `--public-write` | Allow unauthenticated write access |

## List Collections

```bash
# List collections for an API root
taxii-cli collection list --api-root-id a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

Output:
```
ID                                    Title               Alias      Public
──────────────────────────────────────────────────────────────────────────────
86c1741e-7e95-4b17-8940-a8f83eb5fe32  Malware Indicators  -          No
24574d4d-d29a-4b53-80c0-be454dfac6d5  IP Blocklist        blocklist  No
f1e2d3c4-b5a6-7890-abcd-ef1234567890  Public Feed         -          Yes
```

> [!IMPORTANT]
> Note the collection `ID` (UUID) - you'll need this for permissions.

## Set Up Permissions

Permissions are configured in `data-config.yaml` using collection UUIDs:

```yaml
accounts:
  - username: analyst
    password: secret
    is_admin: false
    permissions:
      # TAXII 2.x: use collection UUID
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read, write]
      24574d4d-d29a-4b53-80c0-be454dfac6d5: [read]
```

Apply with:
```bash
taxii-cli sync data-config.yaml
```

> [!TIP]
> For TAXII 2.x only, copy `examples/data-config/accounts.yaml` to `data-config.yaml`.

## Delete Resources

### Delete Collection

```bash
taxii-cli collection delete --id 86c1741e-7e95-4b17-8940-a8f83eb5fe32
```

### Delete API Root

```bash
# Must delete all collections first
taxii-cli api-root delete --id a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

## Complete Setup Example

```bash
# 1. Create API root
taxii-cli api-root add --title "Threat Intel" --default
# Note the API root ID from output

# 2. Create collections
taxii-cli collection add \
  --api-root-id <api-root-id> \
  --title "IOC Feed" \
  --alias iocs

taxii-cli collection add \
  --api-root-id <api-root-id> \
  --title "Malware Analysis" \
  --alias malware

# 3. List to get collection UUIDs
taxii-cli collection list --api-root-id <api-root-id>

# 4. Update data-config.yaml with UUIDs
cat >> data-config.yaml << EOF
accounts:
  - username: analyst
    password: analyst123
    permissions:
      <collection-uuid-1>: [read, write]
      <collection-uuid-2>: [read]
EOF

# 5. Sync accounts
taxii-cli sync data-config.yaml
```

## Verify Setup

Test the discovery endpoint:

```bash
curl http://localhost:9000/taxii2/
```

Response:
```json
{
  "title": "DARWIS TAXII",
  "api_roots": [
    "http://localhost:9000/taxii2/default/"
  ]
}
```

Test collection access:

```bash
TOKEN=$(curl -s -X POST http://localhost:9000/management/auth \
  -H "Content-Type: application/json" \
  -d '{"username": "analyst", "password": "analyst123"}' | jq -r '.token')

curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:9000/taxii2/default/collections/"
```

## Next Steps

- [Collections & API Roots](./collections.md) - Collection properties in detail
- [API Reference](./api.md) - REST API endpoints
- [Permissions](../permissions.md) - Access control
