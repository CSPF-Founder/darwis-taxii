# Server Configuration

DARWIS TAXII server settings are configured via `taxii.toml` or environment variables.

## Configuration File

The server searches for `taxii.toml` in this order:
1. Path specified by `DARWIS_TAXII_CONFIG` env var
2. `./taxii.toml` (current directory)
3. `./config/taxii.toml` (config subdirectory)

### Example Configuration

```toml
bind_address = "0.0.0.0"
port = 9000
domain = "localhost:9000"
support_basic_auth = true
return_server_error_details = false

[database]
url = "postgresql://user:password@localhost:5432/taxii"

[auth]
secret = "your-production-secret-change-this"
token_ttl_secs = 3600

[taxii1]
save_raw_inbox_messages = true
xml_parser_supports_huge_tree = true
count_blocks_in_poll_responses = false
unauthorized_status = "UNAUTHORIZED"

[taxii2]
title = "DARWIS TAXII"
description = "Threat Intelligence Exchange"
contact = "security@example.com"
max_content_length = 104857600
public_discovery = true
allow_custom_properties = true
default_pagination_limit = 1000
max_pagination_limit = 1000
```

## Environment Variables

All settings can be overridden via environment variables with the `DARWIS_TAXII_` prefix.

> [!IMPORTANT]
> Environment variables > TOML config > Defaults

### Required Settings

| Variable | TOML | Description |
|----------|------|-------------|
| `DARWIS_TAXII_DB_CONNECTION` | `database.url` | PostgreSQL connection string |
| `DARWIS_TAXII_AUTH_SECRET` | `auth.secret` | JWT signing secret |

### Server Settings

| Variable | TOML | Default | Description |
|----------|------|---------|-------------|
| `DARWIS_TAXII_CONFIG` | - | `taxii.toml` | Config file path |
| `DARWIS_TAXII_BIND_ADDRESS` | `bind_address` | `0.0.0.0` | Server bind address |
| `DARWIS_TAXII_PORT` | `port` | `9000` | Server port |
| `DARWIS_TAXII_DOMAIN` | `domain` | `localhost:9000` | Public domain for URLs |
| `DARWIS_TAXII_SUPPORT_BASIC_AUTH` | `support_basic_auth` | `true` | Enable HTTP Basic Auth |
| `DARWIS_TAXII_RETURN_SERVER_ERROR_DETAILS` | `return_server_error_details` | `false` | Show error details |

### Auth Settings

| Variable | TOML | Default | Description |
|----------|------|---------|-------------|
| `DARWIS_TAXII_AUTH_SECRET` | `auth.secret` | Required | JWT signing secret |
| `DARWIS_TAXII_TOKEN_TTL_SECS` | `auth.token_ttl_secs` | `3600` | Token lifetime (seconds) |

### TAXII 1.x Settings

| Variable | TOML | Default | Description |
|----------|------|---------|-------------|
| `DARWIS_TAXII_SAVE_RAW_INBOX_MESSAGES` | `taxii1.save_raw_inbox_messages` | `true` | Store original XML |
| `DARWIS_TAXII_XML_PARSER_SUPPORTS_HUGE_TREE` | `taxii1.xml_parser_supports_huge_tree` | `true` | Allow large XML |
| `DARWIS_TAXII_COUNT_BLOCKS_IN_POLL_RESPONSES` | `taxii1.count_blocks_in_poll_responses` | `false` | Include block count |
| `DARWIS_TAXII_UNAUTHORIZED_STATUS` | `taxii1.unauthorized_status` | `UNAUTHORIZED` | Auth failure status |

### TAXII 2.x Settings

| Variable | TOML | Default | Description |
|----------|------|---------|-------------|
| `DARWIS_TAXII_TITLE` | `taxii2.title` | `DARWIS TAXII` | Server title |
| `DARWIS_TAXII_DESCRIPTION` | `taxii2.description` | - | Server description |
| `DARWIS_TAXII_CONTACT` | `taxii2.contact` | - | Contact email |
| `DARWIS_TAXII_PUBLIC_DISCOVERY` | `taxii2.public_discovery` | `true` | Unauthenticated discovery |
| `DARWIS_TAXII_MAX_CONTENT_LENGTH` | `taxii2.max_content_length` | `2048` | Max request body (bytes) |
| `DARWIS_TAXII_ALLOW_CUSTOM_PROPERTIES` | `taxii2.allow_custom_properties` | `true` | Allow custom STIX props |
| `DARWIS_TAXII_DEFAULT_PAGINATION_LIMIT` | `taxii2.default_pagination_limit` | `1000` | Default page size |
| `DARWIS_TAXII_MAX_PAGINATION_LIMIT` | `taxii2.max_pagination_limit` | `1000` | Maximum page size |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

## Database Configuration

### Connection String Format

```
postgresql://username:password@host:port/database
```

### Connection Pool

The server maintains a connection pool to PostgreSQL. Pool size is automatically tuned based on available resources.

### SSL/TLS

For SSL connections:
```
postgresql://user:pass@host:5432/db?sslmode=require
```

## Production Recommendations

### Security

1. **Change the auth secret**:
   ```toml
   [auth]
   secret = "use-a-long-random-string-at-least-32-characters"
   ```

2. **Disable error details**:
   ```toml
   return_server_error_details = false
   ```

3. **Use environment variables for secrets**:
   ```bash
   export DARWIS_TAXII_AUTH_SECRET="your-secret"
   export DARWIS_TAXII_DB_CONNECTION="postgresql://..."
   ```

### Performance

1. **Increase content length for large bundles**:
   ```toml
   [taxii2]
   max_content_length = 104857600  # 100MB
   ```

2. **Tune pagination**:
   ```toml
   [taxii2]
   default_pagination_limit = 100
   max_pagination_limit = 1000
   ```

### Domain Configuration

Set the `domain` to match your public URL:

```toml
domain = "taxii.example.com"
```

This affects:
- Service URLs in TAXII 1.x discovery
- API root URLs in TAXII 2.x discovery

## Docker Configuration

When using Docker, configure via environment variables:

```yaml
# docker-compose.yml
services:
  taxii:
    image: darwis-taxii
    environment:
      - DARWIS_TAXII_DB_CONNECTION=postgresql://user:pass@db:5432/taxii
      - DARWIS_TAXII_AUTH_SECRET=your-secret-here
      - DARWIS_TAXII_DOMAIN=taxii.example.com
      - RUST_LOG=info
    ports:
      - "9000:9000"
```

Or mount a config file:

```yaml
volumes:
  - ./taxii.toml:/app/config/taxii.toml
```
