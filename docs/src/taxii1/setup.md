# TAXII 1.x Setup & Configuration

TAXII 1.x is configured via YAML and applied using the CLI.

## Configuration File

Create a configuration file (e.g., `full.yaml`) with your services, collections, and accounts.
See `examples/data-config/full.yaml` for a complete example:

```yaml
# TAXII 1.x Services
services:
  - id: discovery
    type: DISCOVERY
    properties:
      path: /services/discovery
      description: Discovery service

  - id: inbox
    type: INBOX
    properties:
      path: /services/inbox
      description: Inbox service
      destination_collections:
        - my-collection

  - id: poll
    type: POLL
    properties:
      path: /services/poll
      description: Poll service

  - id: collection-mgmt
    type: COLLECTION_MANAGEMENT
    properties:
      path: /services/collection-management
      description: Collection management

# TAXII 1.x Collections
collections:
  - name: my-collection
    description: Threat intelligence feed
    type: DATA_FEED
    available: true
    accept_all_content: true
    service_ids:
      - inbox
      - poll
      - collection-mgmt
    supported_content:
      - binding: urn:stix.mitre.org:xml:1.1.1
      - binding: urn:stix.mitre.org:xml:1.2

# User accounts
accounts:
  - username: admin
    password: changeme
    is_admin: true

  - username: analyst
    password: secret
    permissions:
      my-collection: modify  # read + write
```

## Apply Configuration

Sync the configuration to the database:

```bash
# From the project directory
taxii-cli sync data-config.yaml

# Or with explicit database connection
DATABASE_URL="postgresql://user:pass@localhost/taxii" taxii-cli sync data-config.yaml
```

Output:
```
Services synchronized: 4 created, 0 updated, 0 deleted
Collections synchronized: 1 created, 0 updated, 0 disabled
Accounts synchronized: 2 created, 0 updated
Configuration synchronized successfully
```

## Configuration Options

### Service Properties

| Property | Description | Required |
|----------|-------------|----------|
| `id` | Unique service identifier | Yes |
| `type` | Service type (see below) | Yes |
| `properties.path` | URL endpoint path | Yes |
| `properties.description` | Human-readable description | No |

### Service Types

| Type | Description |
|------|-------------|
| `DISCOVERY` | Lists available services |
| `INBOX` | Receives content (push) |
| `POLL` | Provides content (pull) |
| `COLLECTION_MANAGEMENT` | Lists collections |

### Inbox-Specific Properties

```yaml
- id: inbox
  type: INBOX
  properties:
    path: /services/inbox
    destination_collections:  # Which collections receive content
      - collection-a
      - collection-b
```

### Poll-Specific Properties

```yaml
- id: poll
  type: POLL
  properties:
    path: /services/poll
    max_result_count: 100     # Max results per response
```

### Collection Properties

| Property | Description | Default |
|----------|-------------|---------|
| `name` | Collection identifier (unique) | Required |
| `description` | Human-readable description | None |
| `type` | `DATA_FEED` or `DATA_SET` | `DATA_FEED` |
| `available` | Is collection active? | `true` |
| `accept_all_content` | Accept any content type? | `true` |
| `service_ids` | Linked services | `[]` |
| `supported_content` | Allowed content bindings | All |

## Update Configuration

To update an existing configuration:

1. Edit `data-config.yaml`
2. Run `taxii-cli sync data-config.yaml` again

The sync command creates and updates entities. To control what happens to entities not in your config file, use YAML-level options:

### Cleanup Options

```yaml
# At the top of your YAML file
prune_services: false            # Delete services not in config
collections_not_in_config: ignore # ignore | disable | delete
prune_accounts: false            # Delete accounts not in config

services:
  # ...
collections:
  # ...
accounts:
  # ...
```

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `prune_services` | `true`/`false` | `false` | Delete services not in config |
| `collections_not_in_config` | `ignore`/`disable`/`delete` | `ignore` | Action for collections not in config |
| `prune_accounts` | `true`/`false` | `false` | Delete accounts not in config |

### Collection Actions

Collections support three cleanup actions:

| Value | Behavior |
|-------|----------|
| `ignore` | Leave untouched (default) |
| `disable` | Set `available=false` |
| `delete` | Permanently delete |

> [!CAUTION]
> `collections_not_in_config: delete` permanently deletes collections and their content.

## Verify Configuration

Check services are configured:

```bash
# Query discovery service
curl -X POST http://localhost:9000/services/discovery \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0" encoding="UTF-8"?>
<Discovery_Request xmlns="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
                   message_id="1"/>'
```

## Next Steps

- [Services](./services.md) - Detailed service configuration
- [Collections](./collections.md) - Collection types and content bindings
- [API Reference](./api.md) - XML message examples
