# TAXII 2.x API Reference

TAXII 2.x is a RESTful API using JSON. All responses use the `application/taxii+json;version=2.1` content type.

## Authentication

```bash
# Get JWT token
TOKEN=$(curl -s -X POST http://localhost:9000/management/auth \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "changeme"}' | jq -r '.token')

# Use token in requests
curl -H "Authorization: Bearer $TOKEN" http://localhost:9000/taxii2/
```

Or use HTTP Basic Auth (if enabled):

```bash
curl -u admin:changeme http://localhost:9000/taxii2/
```

## Discovery

Get server information and available API roots.

**Endpoint:** `GET /taxii2/`

```bash
curl http://localhost:9000/taxii2/
```

**Response:**
```json
{
  "title": "DARWIS TAXII",
  "description": "Threat Intelligence Exchange",
  "contact": "security@example.com",
  "default": "http://localhost:9000/taxii2/default/",
  "api_roots": [
    "http://localhost:9000/taxii2/default/",
    "http://localhost:9000/taxii2/partner-intel/"
  ]
}
```

## API Root Information

Get details about an API root.

**Endpoint:** `GET /taxii2/{api-root}/`

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/
```

**Response:**
```json
{
  "title": "Default API Root",
  "versions": ["application/taxii+json;version=2.1"],
  "max_content_length": 104857600
}
```

## List Collections

Get available collections in an API root.

**Endpoint:** `GET /taxii2/{api-root}/collections/`

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/collections/
```

**Response:**
```json
{
  "collections": [
    {
      "id": "86c1741e-7e95-4b17-8940-a8f83eb5fe32",
      "title": "IOC Feed",
      "description": "Indicators of Compromise",
      "alias": "iocs",
      "can_read": true,
      "can_write": true,
      "media_types": ["application/stix+json;version=2.1"]
    },
    {
      "id": "24574d4d-d29a-4b53-80c0-be454dfac6d5",
      "title": "Malware Analysis",
      "can_read": true,
      "can_write": false,
      "media_types": ["application/stix+json;version=2.1"]
    }
  ]
}
```

## Get Collection

Get details about a specific collection.

**Endpoint:** `GET /taxii2/{api-root}/collections/{collection-id}/`

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/collections/86c1741e-7e95-4b17-8940-a8f83eb5fe32/
```

## Get Objects

Retrieve STIX objects from a collection.

**Endpoint:** `GET /taxii2/{api-root}/collections/{collection-id}/objects/`

### Basic Request

```bash
curl -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/taxii+json;version=2.1" \
  http://localhost:9000/taxii2/default/collections/86c1741e-7e95-4b17-8940-a8f83eb5fe32/objects/
```

### With Filters

```bash
# Filter by object type
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:9000/taxii2/default/collections/<id>/objects/?match[type]=indicator"

# Filter by time range
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:9000/taxii2/default/collections/<id>/objects/?added_after=2024-01-01T00:00:00Z"

# Pagination
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:9000/taxii2/default/collections/<id>/objects/?limit=100"
```

### Query Parameters

| Parameter | Description |
|-----------|-------------|
| `added_after` | Only objects added after this timestamp |
| `match[id]` | Filter by STIX ID |
| `match[type]` | Filter by object type |
| `match[version]` | Filter by version |
| `limit` | Maximum objects to return |
| `next` | Pagination cursor |

**Response:**
```json
{
  "more": false,
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

## Add Objects

Submit STIX objects to a collection.

**Endpoint:** `POST /taxii2/{api-root}/collections/{collection-id}/objects/`

```bash
curl -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/taxii+json;version=2.1" \
  -H "Accept: application/taxii+json;version=2.1" \
  http://localhost:9000/taxii2/default/collections/86c1741e-7e95-4b17-8940-a8f83eb5fe32/objects/ \
  -d '{
    "objects": [
      {
        "type": "indicator",
        "spec_version": "2.1",
        "id": "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f",
        "created": "2024-01-15T12:00:00.000Z",
        "modified": "2024-01-15T12:00:00.000Z",
        "name": "Malicious IP Address",
        "description": "Known C2 server",
        "pattern": "[ipv4-addr:value = '\''198.51.100.1'\'']",
        "pattern_type": "stix",
        "valid_from": "2024-01-15T12:00:00Z"
      }
    ]
  }'
```

**Response:**
```json
{
  "id": "status--2d086da7-4bdc-4f91-900e-d77486753710",
  "status": "complete",
  "total_count": 1,
  "success_count": 1,
  "failure_count": 0,
  "pending_count": 0
}
```

## Get Object

Retrieve a specific STIX object.

**Endpoint:** `GET /taxii2/{api-root}/collections/{collection-id}/objects/{object-id}/`

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/collections/86c1741e-7e95-4b17-8940-a8f83eb5fe32/objects/indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f/
```

## Delete Object

Remove a STIX object from a collection.

**Endpoint:** `DELETE /taxii2/{api-root}/collections/{collection-id}/objects/{object-id}/`

```bash
curl -X DELETE \
  -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/collections/86c1741e-7e95-4b17-8940-a8f83eb5fe32/objects/indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f/
```

## Get Object Versions

List all versions of an object.

**Endpoint:** `GET /taxii2/{api-root}/collections/{collection-id}/objects/{object-id}/versions/`

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/collections/<id>/objects/<object-id>/versions/
```

**Response:**
```json
{
  "more": false,
  "versions": [
    "2024-01-15T12:00:00.000Z",
    "2024-01-16T08:30:00.000Z"
  ]
}
```

## Get Manifest

Get metadata about objects without full content.

**Endpoint:** `GET /taxii2/{api-root}/collections/{collection-id}/manifest/`

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/collections/86c1741e-7e95-4b17-8940-a8f83eb5fe32/manifest/
```

**Response:**
```json
{
  "more": false,
  "objects": [
    {
      "id": "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f",
      "date_added": "2024-01-15T12:00:00.000Z",
      "version": "2024-01-15T12:00:00.000Z",
      "media_type": "application/stix+json;version=2.1"
    }
  ]
}
```

## Job Status

Check status of async operations.

**Endpoint:** `GET /taxii2/{api-root}/status/{status-id}/`

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/taxii2/default/status/status--2d086da7-4bdc-4f91-900e-d77486753710/
```

## Error Responses

Errors return a TAXII error object:

```json
{
  "title": "Unauthorized",
  "description": "Authentication required",
  "http_status": "401"
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created (for POST requests) |
| 400 | Bad request |
| 401 | Authentication required |
| 403 | Forbidden (no permission) |
| 404 | Not found |
| 406 | Not acceptable (wrong Accept header) |
| 415 | Unsupported media type |
| 422 | Unprocessable entity (invalid STIX) |

## Management Endpoints

These endpoints are not part of the TAXII specification but are provided for server management.

### Health Check

**Endpoint:** `GET /management/health`

```bash
curl http://localhost:9000/management/health
```

**Response:**
```json
{"alive": true}
```

### Authentication

**Endpoint:** `POST /management/auth`

```bash
curl -X POST http://localhost:9000/management/auth \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "changeme"}'
```

**Response:**
```json
{"token": "eyJ..."}
```

## TAXII 2.1 Specification

For complete protocol details, see the [TAXII 2.1 Specification](https://docs.oasis-open.org/cti/taxii/v2.1/os/taxii-v2.1-os.html).
