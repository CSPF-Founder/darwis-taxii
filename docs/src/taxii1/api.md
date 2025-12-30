# TAXII 1.x API Reference

TAXII 1.x uses HTTP POST with XML payloads. All endpoints require authentication unless configured otherwise.

## Authentication

```bash
# HTTP Basic Auth
curl -X POST http://localhost:9000/services/discovery \
  -u username:password \
  -H "Content-Type: application/xml" \
  -d '...'

# JWT Bearer Token
curl -X POST http://localhost:9000/services/discovery \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/xml" \
  -d '...'
```

## Discovery Service

Lists available TAXII services.

**Endpoint:** `POST /services/discovery`

**Request:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Discovery_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="1"/>
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Discovery_Response
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="1"
    in_response_to="1">
  <taxii_11:Service_Instance service_type="DISCOVERY" available="true">
    <taxii_11:Protocol_Binding>urn:taxii.mitre.org:protocol:http:1.0</taxii_11:Protocol_Binding>
    <taxii_11:Address>http://localhost:9000/services/discovery</taxii_11:Address>
    <taxii_11:Message_Binding>urn:taxii.mitre.org:message:xml:1.1</taxii_11:Message_Binding>
  </taxii_11:Service_Instance>
  <taxii_11:Service_Instance service_type="INBOX" available="true">
    <taxii_11:Protocol_Binding>urn:taxii.mitre.org:protocol:http:1.0</taxii_11:Protocol_Binding>
    <taxii_11:Address>http://localhost:9000/services/inbox</taxii_11:Address>
    <taxii_11:Message_Binding>urn:taxii.mitre.org:message:xml:1.1</taxii_11:Message_Binding>
  </taxii_11:Service_Instance>
  <!-- Additional services... -->
</taxii_11:Discovery_Response>
```

## Collection Management

Lists available collections.

**Endpoint:** `POST /services/collection-management`

**Request:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Collection_Information_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="2"/>
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Collection_Information_Response
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="2"
    in_response_to="2">
  <taxii_11:Collection collection_name="my-collection"
                       collection_type="DATA_FEED"
                       available="true">
    <taxii_11:Description>Threat intelligence feed</taxii_11:Description>
    <taxii_11:Content_Binding binding_id="urn:stix.mitre.org:xml:1.1.1"/>
    <taxii_11:Polling_Service>
      <taxii_11:Protocol_Binding>urn:taxii.mitre.org:protocol:http:1.0</taxii_11:Protocol_Binding>
      <taxii_11:Address>http://localhost:9000/services/poll</taxii_11:Address>
    </taxii_11:Polling_Service>
  </taxii_11:Collection>
</taxii_11:Collection_Information_Response>
```

## Poll Service

Retrieves content from a collection.

**Endpoint:** `POST /services/poll`

### Basic Poll Request

```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Poll_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="3"
    collection_name="my-collection"/>
```

### Poll with Time Range

```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Poll_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="3"
    collection_name="my-collection">
  <taxii_11:Exclusive_Begin_Timestamp>2024-01-01T00:00:00Z</taxii_11:Exclusive_Begin_Timestamp>
  <taxii_11:Inclusive_End_Timestamp>2024-01-31T23:59:59Z</taxii_11:Inclusive_End_Timestamp>
</taxii_11:Poll_Request>
```

### Poll Response

```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Poll_Response
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="3"
    in_response_to="3"
    collection_name="my-collection">
  <taxii_11:Content_Block>
    <taxii_11:Content_Binding binding_id="urn:stix.mitre.org:xml:1.1.1"/>
    <taxii_11:Content>
      <!-- STIX content here -->
    </taxii_11:Content>
    <taxii_11:Timestamp_Label>2024-01-15T10:30:00Z</taxii_11:Timestamp_Label>
  </taxii_11:Content_Block>
</taxii_11:Poll_Response>
```

## Inbox Service

Submits content to a collection.

**Endpoint:** `POST /services/inbox`

**Request:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Inbox_Message
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="4">
  <taxii_11:Destination_Collection_Name>my-collection</taxii_11:Destination_Collection_Name>
  <taxii_11:Content_Block>
    <taxii_11:Content_Binding binding_id="urn:stix.mitre.org:xml:1.1.1"/>
    <taxii_11:Content>
      <stix:STIX_Package xmlns:stix="http://stix.mitre.org/stix-1"
                         id="example:Package-1"
                         version="1.1.1"
                         timestamp="2024-01-15T10:30:00Z">
        <stix:Indicators>
          <stix:Indicator id="example:indicator-1" timestamp="2024-01-15T10:30:00Z">
            <indicator:Title>Malicious IP Address</indicator:Title>
          </stix:Indicator>
        </stix:Indicators>
      </stix:STIX_Package>
    </taxii_11:Content>
  </taxii_11:Content_Block>
</taxii_11:Inbox_Message>
```

**Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Status_Message
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="4"
    in_response_to="4"
    status_type="SUCCESS"/>
```

## cURL Examples

### Discovery

```bash
curl -X POST http://localhost:9000/services/discovery \
  -u admin:changeme \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0"?>
<taxii_11:Discovery_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="1"/>'
```

### List Collections

```bash
curl -X POST http://localhost:9000/services/collection-management \
  -u admin:changeme \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0"?>
<taxii_11:Collection_Information_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="1"/>'
```

### Poll Collection

```bash
curl -X POST http://localhost:9000/services/poll \
  -u admin:changeme \
  -H "Content-Type: application/xml" \
  -d '<?xml version="1.0"?>
<taxii_11:Poll_Request
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="1"
    collection_name="my-collection"/>'
```

## Error Responses

Errors return a Status_Message with status_type indicating the error:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<taxii_11:Status_Message
    xmlns:taxii_11="http://taxii.mitre.org/messages/taxii_xml_binding-1.1"
    message_id="1"
    in_response_to="1"
    status_type="UNAUTHORIZED">
  <taxii_11:Message>Authentication required</taxii_11:Message>
</taxii_11:Status_Message>
```

### Status Types

| Status | Description |
|--------|-------------|
| `SUCCESS` | Request completed successfully |
| `UNAUTHORIZED` | Authentication failed |
| `NOT_FOUND` | Collection or resource not found |
| `DESTINATION_COLLECTION_ERROR` | Invalid destination collection |
| `INVALID_REQUEST` | Malformed request |

## TAXII 1.1 Specification

For complete protocol details, see the [TAXII 1.1.1 Specification](https://docs.oasis-open.org/cti/taxii/v1.1.1/taxii-v1.1.1-part1-overview.html).
