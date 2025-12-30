# TAXII 1.x Collections

Collections are containers for threat intelligence content in TAXII 1.x.

## Collection Identifier

In TAXII 1.x, the collection `name` is the identifier. There are no UUIDs.

```yaml
collections:
  - name: my-collection    # This IS the identifier
    description: My threat intelligence
```

This name is used:
- In poll requests to specify which collection to query
- In inbox messages to specify destination
- In permissions to grant access

## Collection Types

### DATA_FEED

One-way data flow. Content is produced by the server and consumed by clients.

```yaml
- name: threat-feed
  type: DATA_FEED
  service_ids:
    - poll           # Clients can pull
    # No inbox - clients cannot push
```

Use for:
- External threat feeds
- Read-only intelligence
- Broadcast content

### DATA_SET

Bidirectional data flow. Clients can both push and pull content.

```yaml
- name: shared-intel
  type: DATA_SET
  service_ids:
    - inbox          # Clients can push
    - poll           # Clients can pull
```

Use for:
- Collaborative intelligence sharing
- Community collections
- Internal threat repositories

## Content Bindings

Content bindings restrict what types of content a collection accepts.

### Accept All Content

```yaml
- name: open-collection
  accept_all_content: true    # Any content type allowed
```

### Restrict Content Types

```yaml
- name: stix-only
  accept_all_content: false
  supported_content:
    - binding: urn:stix.mitre.org:xml:1.1.1
    - binding: urn:stix.mitre.org:xml:1.2
```

### Common Content Bindings

| Binding URN | Description |
|-------------|-------------|
| `urn:stix.mitre.org:xml:1.0` | STIX 1.0 |
| `urn:stix.mitre.org:xml:1.1.1` | STIX 1.1.1 |
| `urn:stix.mitre.org:xml:1.2` | STIX 1.2 |

### Subtypes

Content bindings can have subtypes:

```yaml
supported_content:
  - binding: urn:stix.mitre.org:xml:1.1.1
    subtypes:
      - stix.mitre.org:xml:1.1.1:indicator
      - stix.mitre.org:xml:1.1.1:ttp
```

## Service Linkage

Collections must be linked to services to be accessible:

```yaml
collections:
  - name: intel-feed
    service_ids:
      - inbox              # Receive via inbox service
      - poll               # Query via poll service
      - collection-mgmt    # List via collection management
```

### Access Pattern Examples

**Read-only collection:**
```yaml
- name: external-feed
  service_ids:
    - poll
    - collection-mgmt
```

**Write-only collection:**
```yaml
- name: submissions
  service_ids:
    - inbox
```

**Full access:**
```yaml
- name: shared-intel
  service_ids:
    - inbox
    - poll
    - collection-mgmt
```

## Availability

Control whether a collection is active:

```yaml
- name: my-collection
  available: true     # Active - accessible via services
```

```yaml
- name: archived
  available: false    # Inactive - not accessible
```

Setting `available: false`:
- Hides the collection from discovery
- Rejects inbox/poll requests
- Preserves the data

## Full Example

```yaml
collections:
  # Primary threat intelligence feed
  - name: threat-intel
    description: Curated threat intelligence
    type: DATA_FEED
    available: true
    accept_all_content: false
    supported_content:
      - binding: urn:stix.mitre.org:xml:1.1.1
      - binding: urn:stix.mitre.org:xml:1.2
    service_ids:
      - poll
      - collection-mgmt

  # Community sharing collection
  - name: community-sharing
    description: Community-contributed intelligence
    type: DATA_SET
    available: true
    accept_all_content: true
    service_ids:
      - inbox
      - poll
      - collection-mgmt

  # Internal submissions
  - name: internal-submissions
    description: Internal threat submissions
    type: DATA_SET
    available: true
    accept_all_content: true
    service_ids:
      - inbox
```

## Managing Collections

### List Collections

Query the collection management service:

```bash
curl -X POST http://localhost:9000/services/collection-management \
  -u analyst:password \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0"?>
<taxii_11:Collection_Information_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="1"/>'
```

### Update Collections

1. Edit `data-config.yaml`
2. Run `taxii-cli sync data-config.yaml`

### Delete Collections

Remove from config and sync with `--force-delete`:

```bash
taxii-cli sync data-config.yaml --force-delete
```

## Next Steps

- [API Reference](./api.md) - XML message examples
- [Permissions](../permissions.md) - Collection access control
