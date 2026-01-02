# Quick Start

This guide covers the basic steps to install and run DARWIS TAXII.

## Prerequisites

- PostgreSQL 9.4+ (minimal version to ensure compatibility with existing OpenTAXII instances)
- Docker (optional, for containerized deployment)

## Option 1: Docker from Docker Hub (Recommended)

Pull and run directly from [Docker Hub](https://hub.docker.com/r/cysecurity/darwis-taxii):

```bash
# Pull the latest image
docker pull cysecurity/darwis-taxii:latest

# Create a working directory
mkdir -p darwis-taxii/config && cd darwis-taxii

# Download example configuration files
curl -o config/taxii.toml https://raw.githubusercontent.com/CSPF-Founder/darwis-taxii/main/taxii.example.toml
curl -o config/data-config.yaml https://raw.githubusercontent.com/CSPF-Founder/darwis-taxii/main/examples/data-config/full.yaml
curl -o docker-compose.yml https://raw.githubusercontent.com/CSPF-Founder/darwis-taxii/main/examples/docker/docker-compose.yml

# Edit configuration files as needed (optional)
# - config/taxii.toml: server settings, domain, auth options
# - config/data-config.yaml: services, collections, accounts

# Start the server with PostgreSQL
docker compose up -d

# Sync data configuration
docker compose exec taxii-server ./taxii-cli sync /app/config/data-config.yaml

# Verify it's running
curl http://localhost:9000/taxii2/
```

## Option 2: Docker from Source

```bash
# Clone the repository
git clone https://github.com/CSPF-Founder/darwis-taxii.git
cd darwis-taxii/examples/docker

# Create configuration
mkdir -p config
cp ../data-config/full.yaml config/data-config.yaml  # Or accounts.yaml for TAXII 2.x only

# Start the server
docker compose up -d

# Verify it's running
curl http://localhost:9000/taxii2/
```

## Option 3: From Source

```bash
# Clone and build
git clone https://github.com/CSPF-Founder/darwis-taxii.git
cd darwis-taxii
cargo build --release

# Set up database
export DATABASE_URL="postgresql://user:password@localhost:5432/taxii"
./target/release/taxii-server migrate

# Create configuration
cp taxii.example.toml taxii.toml
# Edit taxii.toml with your settings

# Start the server
./target/release/taxii-server
```

## Verify Installation

### Health Check

```bash
curl http://localhost:9000/management/health
```

Response:
```json
{"alive": true}
```

### Test TAXII 2.x Discovery

```bash
curl http://localhost:9000/taxii2/
```

Response:
```json
{
  "title": "DARWIS TAXII",
  "api_roots": ["http://localhost:9000/taxii2/default/"]
}
```

### Get Authentication Token

```bash
curl -X POST http://localhost:9000/management/auth \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "changeme"}'
```

Response:
```json
{"token": "eyJ..."}
```

## Next Steps

1. **Configure TAXII 1.x** - [TAXII 1.x Setup](./taxii1/setup.md)
   - Copy template: `cp examples/data-config/full.yaml data-config.yaml`
   - Sync with `taxii-cli sync data-config.yaml`

2. **Configure TAXII 2.x** - [TAXII 2.x Setup](./taxii2/setup.md)
   - Create API roots: `taxii-cli api-root add --title "My Root" --default`
   - Create collections: `taxii-cli collection add --api-root-id <uuid> --title "Intel"`

3. **Set up user accounts** - [Permissions](./permissions.md)
   - Define accounts in `data-config.yaml`
   - Assign permissions to collections

4. **Configure the server** - [Server Configuration](./configuration.md)
   - Edit `taxii.toml` for server settings
   - Use environment variables for production

## Common Commands

```bash
# List accounts
taxii-cli account list

# List TAXII 2.x API roots
taxii-cli api-root list

# List collections for an API root
taxii-cli collection list --api-root-id <uuid>

# Sync configuration
taxii-cli sync data-config.yaml
```
