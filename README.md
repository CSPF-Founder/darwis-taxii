# DARWIS TAXII

A high-performance TAXII (Trusted Automated eXchange of Indicator Information) server written in Rust.

Rust port of [EclecticIQ OpenTAXII](https://github.com/EclecticIQ/OpenTAXII) with full database compatibility.

## Features

- **TAXII 1.x** - Discovery, Poll, Inbox, Collection Management, Subscriptions
- **TAXII 2.1** - Full REST API with Collections, Objects, Manifest, Versions, Status
- **STIX 2.x** - Native parsing and validation
- **Authentication** - JWT tokens and Basic Auth with per-collection permissions
- **Async** - Built on Tokio and Axum for high throughput
- **Extensible** - Signal hooks for custom integrations

## Requirements

- Rust 1.85+ (2024 edition)
- PostgreSQL 9.4+

## Quick Start

```bash
# Build
cargo build --release

# Copy and edit configuration
cp taxii.example.toml taxii.toml
cp examples/data-config/full.yaml data-config.yaml  # Or accounts.yaml for TAXII 2.x only
# Edit taxii.toml and data-config.yaml with your settings

# Run server (auto-runs migrations on startup)
./target/release/taxii-server

# Sync data configuration (services, collections, accounts)
./target/release/taxii-cli sync data-config.yaml
```

Server starts at `http://localhost:9000`.

## Docker

Docker files are in `examples/docker/`:

```bash
cd examples/docker

# Create config directory and copy example configs
mkdir config
cp ../../taxii.example.toml config/taxii.toml
cp ../data-config/full.yaml config/data-config.yaml  # Or accounts.yaml for TAXII 2.x only
# Edit config files as needed

# Start server and PostgreSQL
docker compose up -d

# Sync data configuration (services, collections, accounts)
docker compose exec taxii-server ./taxii-cli sync /app/config/data-config.yaml

# List accounts
docker compose exec taxii-server ./taxii-cli account list
```

Build image only:

```bash
docker build -f examples/docker/Dockerfile -t darwis-taxii .
```

## Configuration

DARWIS TAXII uses two configuration files (matching OpenTAXII convention):

| File | Purpose |
|------|---------|
| [`taxii.toml`](taxii.example.toml) | Server settings: database, auth, domain, TAXII 1.x/2.x options |
| [`data-config/`](examples/data-config/) | Data: services, collections, accounts with permissions |

Copy `taxii.example.toml` to `taxii.toml` and edit for your environment.

See the [documentation](https://cspf-founder.github.io/darwis-taxii/) for all options including environment variable overrides.

## CLI

```bash
# Sync services, collections, and accounts from YAML
taxii-cli sync data-config.yaml

# Account management
taxii-cli account list              # List accounts with permissions
taxii-cli account delete -u user    # Delete an account

# Database migrations (auto-run on server startup)
taxii-cli migrate run               # Apply pending migrations
taxii-cli migrate status            # Show migration status

# TAXII 2.x management
taxii-cli api-root add --title "My Root" --default
taxii-cli collection add --api-root-id <uuid> --title "Intel"
```

Run `taxii-cli --help` for all commands.

## Project Structure

```
taxii.example.toml       # Server configuration example
examples/
  data-config/             # Data configuration examples
    full.yaml              # Full config (TAXII 1.x + accounts)
    accounts.yaml          # Accounts only (for TAXII 2.x)
  docker/                  # Docker deployment files
migrations/              # SQLx database migrations
taxii-core/              # Core types and signal hooks
taxii-db/                # Database persistence (SQLx)
taxii-auth/              # JWT and password authentication
taxii-1x/                # TAXII 1.x protocol (XML)
taxii-2x/                # TAXII 2.x protocol (JSON)
stix2/                   # STIX 2.x parsing
taxii-server/            # HTTP server (Axum)
taxii-cli/               # CLI tool
```

## Documentation

**https://cspf-founder.github.io/darwis-taxii/**

## Compatibility

- **OpenTAXII** - Same database schema, werkzeug password hashes
- **TAXII** - 1.0, 1.1, 2.0, 2.1 specifications
- **STIX** - 2.0, 2.1 objects

## License

BSD-3-Clause

## Acknowledgments

- [EclecticIQ OpenTAXII](https://github.com/EclecticIQ/OpenTAXII) - Original Python implementation
- [OASIS CTI](https://oasis-open.github.io/cti-documentation/) - TAXII/STIX specifications
