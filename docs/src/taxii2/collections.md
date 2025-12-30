# TAXII 2.x Collections & API Roots

Understanding the TAXII 2.x resource hierarchy.

## API Roots

API roots are top-level groupings for collections.

### Properties

| Property | Description |
|----------|-------------|
| `id` | UUID identifier |
| `title` | Human-readable name |
| `description` | Optional description |
| `default` | Whether this is the default API root |

### Use Cases

- **By Department**: "SOC", "Threat Intel", "Incident Response"
- **By Access Level**: "Public", "Partners", "Internal"
- **By Data Type**: "Malware", "Phishing", "Network IOCs"

### Default API Root

The default API root is accessible at `/taxii2/default/`:

```bash
taxii-cli api-root add --title "Primary Intel" --default
```

Other API roots use their title (slugified):
- "Threat Intel" → `/taxii2/threat-intel/`

## Collections

Collections store STIX 2.x objects.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | UUID | **The identifier** - use this for permissions |
| `title` | String | Human-readable name (NOT unique) |
| `description` | String | Optional description |
| `alias` | String | URL-friendly name (unique within API root) |
| `is_public` | Boolean | Allow unauthenticated read access |
| `is_public_write` | Boolean | Allow unauthenticated write access |

### Collection ID vs Title

**The `id` (UUID) is the canonical identifier:**

```
86c1741e-7e95-4b17-8940-a8f83eb5fe32
```

**The `title` is for display only:**
- Multiple collections can have the same title
- Titles can be changed without affecting integrations
- Never use title for permissions

**Example:**
```
API Root: "Threat Intel"
├── Collection "IOCs" (id: 86c1741e-...)
├── Collection "IOCs" (id: 24574d4d-...)  ← Same title, different collection!
└── Collection "Malware" (id: f1e2d3c4-...)
```

### Using Alias

Alias provides a friendly URL alternative to UUID:

```bash
taxii-cli collection add \
  --api-root-id <uuid> \
  --title "IP Blocklist" \
  --alias blocklist
```

Access options:
- By UUID: `/taxii2/default/collections/86c1741e-.../`
- By alias: `/taxii2/default/collections/blocklist/`

> [!NOTE]
> Alias must be unique within an API root but can duplicate across API roots.

## Access Control

### Public Collections

Public collections allow unauthenticated access:

```bash
# Read-only public
taxii-cli collection add \
  --api-root-id <uuid> \
  --title "Public Feed" \
  --public

# Fully public (not recommended)
taxii-cli collection add \
  --api-root-id <uuid> \
  --title "Open Submission" \
  --public \
  --public-write
```

### Permission-Based Access

For non-public collections, configure permissions per user:

```yaml
accounts:
  - username: analyst
    permissions:
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read]        # Read only
      24574d4d-d29a-4b53-80c0-be454dfac6d5: [read, write] # Full access
```

## Collection Endpoints

Each collection provides these endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/collections/{id}/` | GET | Collection info |
| `/collections/{id}/objects/` | GET | List objects |
| `/collections/{id}/objects/` | POST | Add objects |
| `/collections/{id}/objects/{object_id}/` | GET | Get specific object |
| `/collections/{id}/objects/{object_id}/` | DELETE | Delete object |
| `/collections/{id}/manifest/` | GET | Object metadata only |

## Finding Collection UUIDs

### Via CLI

```bash
taxii-cli collection list --api-root-id <api-root-uuid>
```

### Via API

```bash
curl http://localhost:9000/taxii2/default/collections/ \
  -H "Authorization: Bearer $TOKEN"
```

Response:
```json
{
  "collections": [
    {
      "id": "86c1741e-7e95-4b17-8940-a8f83eb5fe32",
      "title": "IOC Feed",
      "can_read": true,
      "can_write": true,
      "media_types": ["application/stix+json;version=2.1"]
    }
  ]
}
```

## Collection Management

### Update Collection (Not Supported)

Currently, collections cannot be updated after creation. Delete and recreate if changes are needed.

### Delete Collection

```bash
taxii-cli collection delete --id 86c1741e-7e95-4b17-8940-a8f83eb5fe32
```

> [!CAUTION]
> This deletes all objects in the collection.

### Move Objects Between Collections

Not directly supported. Export objects and re-import to new collection.

## Best Practices

1. **Use meaningful titles** - But remember they're for humans, not code
2. **Use aliases for stable URLs** - If you share URLs with partners
3. **Note UUIDs immediately** - Copy them when you create collections
4. **One purpose per collection** - Don't mix unrelated data
5. **Consider API root structure** - Group related collections logically

## Next Steps

- [API Reference](./api.md) - REST API examples
- [Permissions](../permissions.md) - Access control details
