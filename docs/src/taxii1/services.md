# TAXII 1.x Services

TAXII 1.x uses four service types to organize threat intelligence exchange.

## Service Types

### Discovery Service

The entry point for TAXII 1.x clients. Returns information about available services.

```yaml
services:
  - id: discovery
    type: DISCOVERY
    properties:
      path: /services/discovery
      description: Discovery service
```

Clients query this service first to learn what other services are available.

### Inbox Service

Receives threat intelligence content (push model). Clients send STIX content to store in collections.

```yaml
services:
  - id: inbox
    type: INBOX
    properties:
      path: /services/inbox
      description: Inbox service for receiving content
      destination_collections:
        - collection-a
        - collection-b
```

**Properties:**

| Property | Description |
|----------|-------------|
| `destination_collections` | Collections that receive inbox content |

When content is pushed to the inbox, it's stored in the specified destination collections.

### Poll Service

Provides threat intelligence content (pull model). Clients request content from collections.

```yaml
services:
  - id: poll
    type: POLL
    properties:
      path: /services/poll
      description: Poll service for retrieving content
      max_result_count: 100
```

**Properties:**

| Property | Description | Default |
|----------|-------------|---------|
| `max_result_count` | Maximum results per response | Unlimited |

Clients can filter poll requests by:
- Collection name
- Time range (exclusive_begin_timestamp, inclusive_end_timestamp)
- Content bindings

### Collection Management Service

Lists available collections and their properties.

```yaml
services:
  - id: collection-mgmt
    type: COLLECTION_MANAGEMENT
    properties:
      path: /services/collection-management
      description: Collection management service
```

Returns information about collections the client has access to.

## Service-Collection Linkage

Collections are linked to services via `service_ids`:

```yaml
collections:
  - name: my-collection
    service_ids:
      - inbox      # Can receive content via inbox
      - poll       # Can provide content via poll
      - collection-mgmt  # Listed in collection management
```

A collection can be:
- Push-only: Only linked to `inbox`
- Pull-only: Only linked to `poll`
- Bidirectional: Linked to both `inbox` and `poll`

## URL Endpoints

Services are accessible at their configured paths:

| Service | Default Path | Method |
|---------|--------------|--------|
| Discovery | `/services/discovery` | POST |
| Inbox | `/services/inbox` | POST |
| Poll | `/services/poll` | POST |
| Collection Management | `/services/collection-management` | POST |

All TAXII 1.x endpoints use HTTP POST with XML payloads.

## Multiple Services

You can define multiple services of the same type:

```yaml
services:
  # High-priority inbox
  - id: inbox-priority
    type: INBOX
    properties:
      path: /services/inbox-priority
      destination_collections:
        - priority-intel

  # Standard inbox
  - id: inbox-standard
    type: INBOX
    properties:
      path: /services/inbox
      destination_collections:
        - general-intel
```

## Service Discovery Response

When a client queries the discovery service, they receive:

```xml
<Discovery_Response xmlns="http://taxii.mitre.org/messages/taxii_xml_binding-1.1">
  <Service_Instance service_type="DISCOVERY" available="true">
    <Protocol_Binding>urn:taxii.mitre.org:protocol:http:1.0</Protocol_Binding>
    <Address>http://localhost:9000/services/discovery</Address>
  </Service_Instance>
  <Service_Instance service_type="INBOX" available="true">
    <Protocol_Binding>urn:taxii.mitre.org:protocol:http:1.0</Protocol_Binding>
    <Address>http://localhost:9000/services/inbox</Address>
  </Service_Instance>
  <!-- ... other services ... -->
</Discovery_Response>
```

## Next Steps

- [Collections](./collections.md) - Collection configuration
- [API Reference](./api.md) - XML message examples
