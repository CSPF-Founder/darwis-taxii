# TAXII 1.x Overview

TAXII 1.x is an XML-based protocol for exchanging cyber threat intelligence. It uses a service-oriented architecture with four main service types.

## Protocol Characteristics

| Aspect | Description |
|--------|-------------|
| **Transport** | HTTP POST with XML payloads |
| **Content** | STIX 1.x (XML format) |
| **Collection ID** | `name` (string identifier) |
| **Configuration** | YAML file + `taxii-cli sync` |

## Service Architecture

TAXII 1.x organizes threat intelligence exchange through four service types:

```
┌─────────────────────────────────────────────────────────────┐
│                     TAXII 1.x Services                      │
├────────────────┬────────────────┬────────────────┬──────────┤
│   Discovery    │     Inbox      │      Poll      │ CollMgmt │
│                │                │                │          │
│ List services  │ Push content   │ Pull content   │ List     │
│ available      │ TO server      │ FROM server    │ colls    │
└────────────────┴────────────────┴────────────────┴──────────┘
         │                │                │              │
         └────────────────┴────────────────┴──────────────┘
                                  │
                          ┌───────┴───────┐
                          │  Collections  │
                          │  (by name)    │
                          └───────────────┘
```

### Discovery Service

Provides information about available services. Clients query this endpoint first to learn what services are available.

### Inbox Service

Receives threat intelligence content (push model). Clients send STIX content to this service to store it in collections.

### Poll Service

Provides threat intelligence content (pull model). Clients request content from collections, optionally filtering by time range.

### Collection Management Service

Lists available collections and their properties.

## Collections

TAXII 1.x collections store threat intelligence content. Each collection:

- Has a unique `name` (the identifier)
- Can be linked to multiple services
- Supports content type filtering
- Has availability status

### Collection Types

| Type | Description |
|------|-------------|
| **DATA_FEED** | One-way data flow (server → client) |
| **DATA_SET** | Bidirectional (clients can push and pull) |

## Content Bindings

Content bindings specify what types of content a collection accepts:

```yaml
supported_content:
  - binding: urn:stix.mitre.org:xml:1.1.1
  - binding: urn:stix.mitre.org:xml:1.2
```

If `accept_all_content: true`, all content types are accepted.

## Configuration Method

TAXII 1.x services and collections are configured via YAML:

1. Copy template: `cp examples/data-config/full.yaml data-config.yaml`
2. Apply with `taxii-cli sync data-config.yaml`

See [TAXII 1.x Setup](./setup.md) for detailed configuration.

## Permissions

TAXII 1.x permissions use the collection `name` as the key:

```yaml
accounts:
  - username: analyst
    password: secret
    permissions:
      my-collection: read       # Read-only
      other-collection: modify  # Read + write
```

See [Permissions](../permissions.md) for details.

## Next Steps

- [Setup & Configuration](./setup.md) - Configure TAXII 1.x services
- [Services](./services.md) - Service types in detail
- [Collections](./collections.md) - Collection configuration
- [API Reference](./api.md) - XML endpoint examples
