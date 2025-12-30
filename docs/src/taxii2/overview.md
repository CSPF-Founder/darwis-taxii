# TAXII 2.x Overview

TAXII 2.x is a RESTful JSON API for exchanging cyber threat intelligence. It provides a modern, standards-based approach to threat intelligence sharing.

## Protocol Characteristics

| Aspect | Description |
|--------|-------------|
| **Transport** | RESTful HTTP with JSON payloads |
| **Content** | STIX 2.x (JSON format) |
| **Collection ID** | UUID (globally unique identifier) |
| **Configuration** | CLI commands |

## Architecture

TAXII 2.x organizes resources in a hierarchy:

```
┌──────────────────────────────────────────────────────────────┐
│                   TAXII 2.x Server                           │
│                                                              │
│  Discovery (/taxii2/)                                        │
│      │                                                       │
│      ├── API Root (/taxii2/default/)                         │
│      │       │                                               │
│      │       ├── Collections                                 │
│      │       │       ├── Collection A (UUID)                 │
│      │       │       │       ├── Objects (STIX)              │
│      │       │       │       └── Manifest                    │
│      │       │       └── Collection B (UUID)                 │
│      │       │               ├── Objects (STIX)              │
│      │       │               └── Manifest                    │
│      │       └── Status (Job tracking)                       │
│      │                                                       │
│      └── API Root (/taxii2/other-root/)                      │
│              └── Collections...                              │
└──────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Discovery

The `/taxii2/` endpoint provides server information and lists available API roots.

### API Roots

API roots are logical groupings of collections. Each API root:
- Has its own URL path (e.g., `/taxii2/default/`)
- Can contain multiple collections
- May have different access controls

### Collections

Collections store STIX objects. Each collection:
- Has a UUID identifier (the `id` field)
- Has a human-readable `title`
- May have an optional `alias`
- Contains STIX 2.x objects

> [!IMPORTANT]
> The collection `id` (UUID) is the canonical identifier. The `title` and `alias` are for display purposes only.

### Objects

STIX 2.x objects are JSON documents representing threat intelligence:
- Indicators (IOCs)
- Malware descriptions
- Threat actors
- Attack patterns
- And more...

## Collection Identifier

In TAXII 2.x, collections are identified by their UUID:

```
86c1741e-7e95-4b17-8940-a8f83eb5fe32
```

This UUID is used:
- In API URLs to access the collection
- In permissions to grant access
- In all programmatic interactions

The `title` is NOT the identifier:
- Two collections can have the same title
- Titles can change
- Always use the UUID for permissions

## Configuration Method

TAXII 2.x is configured via CLI commands (not YAML sync):

```bash
# Create an API root
taxii-cli api-root add --title "Intel" --default

# Create a collection
taxii-cli collection add --api-root-id <uuid> --title "Threat Feed"
```

See [TAXII 2.x Setup](./setup.md) for detailed configuration.

## Permissions

TAXII 2.x permissions use the collection UUID:

```yaml
accounts:
  - username: analyst
    password: secret
    permissions:
      # Use the collection UUID, not title
      86c1741e-7e95-4b17-8940-a8f83eb5fe32: [read, write]
```

Find collection UUIDs with:
```bash
taxii-cli collection list --api-root-id <api-root-uuid>
```

See [Permissions](../permissions.md) for details.

## STIX 2.x Content

TAXII 2.x transports STIX 2.x objects as JSON bundles:

```json
{
  "type": "bundle",
  "id": "bundle--example",
  "objects": [
    {
      "type": "indicator",
      "spec_version": "2.1",
      "id": "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f",
      "created": "2024-01-15T12:00:00.000Z",
      "modified": "2024-01-15T12:00:00.000Z",
      "name": "Malicious IP",
      "pattern": "[ipv4-addr:value = '198.51.100.1']",
      "pattern_type": "stix",
      "valid_from": "2024-01-15T12:00:00Z"
    }
  ]
}
```

## Comparison with TAXII 1.x

| Aspect | TAXII 1.x | TAXII 2.x |
|--------|-----------|-----------|
| Format | XML | JSON |
| Style | Service-oriented | RESTful |
| Operations | INBOX (push), POLL (pull) | POST/GET on endpoints |
| Collection ID | Name (string) | UUID |
| Configuration | YAML + sync | CLI commands |
| Content | STIX 1.x | STIX 2.x |

## Next Steps

- [Setup & Configuration](./setup.md) - Create API roots and collections
- [Collections & API Roots](./collections.md) - Understanding the hierarchy
- [API Reference](./api.md) - REST API endpoints
