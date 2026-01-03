# Introduction

DARWIS TAXII is an implementation of the TAXII (Trusted Automated eXchange of Intelligence Information) protocol, written in Rust. It is designed as a drop-in replacement for [EclecticIQ OpenTAXII](https://github.com/EclecticIQ/OpenTAXII) (Python), maintaining full database compatibility.

**Docker Hub**: [`cysecurity/darwis-taxii`](https://hub.docker.com/r/cysecurity/darwis-taxii)

## What is TAXII?

TAXII is an application protocol for exchanging cyber threat intelligence (CTI) over HTTPS. It defines a set of services and message exchanges for sharing actionable threat information between organizations.

## Supported Protocols

DARWIS TAXII supports both major versions of the TAXII specification:

| Version | Specification | Transport | Content Format |
|---------|--------------|-----------|----------------|
| **TAXII 1.x** | [TAXII 1.1.1](https://docs.oasis-open.org/cti/taxii/v1.1.1/taxii-v1.1.1-part1-overview.html) | HTTP POST with XML | STIX 1.x (XML) |
| **TAXII 2.x** | [TAXII 2.1](https://docs.oasis-open.org/cti/taxii/v2.1/os/taxii-v2.1-os.html) | RESTful JSON | STIX 2.x (JSON) |

## Key Features

- **Dual Protocol Support**: Run TAXII 1.x and 2.x simultaneously on the same server
- **Database Compatible**: Uses the same PostgreSQL schema as OpenTAXII
- **Password Compatible**: Supports werkzeug (scrypt) password hashes from OpenTAXII
- **CLI Management**: Command-line interface for administration

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      DARWIS TAXII Server                    │
├─────────────────────────────┬───────────────────────────────┤
│        TAXII 1.x            │          TAXII 2.x            │
│  ┌───────────────────────┐  │  ┌─────────────────────────┐  │
│  │ Discovery Service     │  │  │ /taxii2/                │  │
│  │ Inbox Service         │  │  │ /taxii2/{api-root}/     │  │
│  │ Poll Service          │  │  │ /taxii2/.../collections │  │
│  │ Collection Management │  │  │ /taxii2/.../objects     │  │
│  └───────────────────────┘  │  └─────────────────────────┘  │
│            │                │              │                │
│   data_collections table    │   opentaxii_collection table  │
│      (separate storage)     │       (separate storage)      │
├─────────────────────────────┴───────────────────────────────┤
│                     PostgreSQL Database                     │
└─────────────────────────────────────────────────────────────┘
```

> [!NOTE]
> TAXII 1.x and TAXII 2.x use **separate collection storage**. Collections cannot be shared between protocols.

## When to Use Each Protocol

**Use TAXII 1.x when:**
- Integrating with legacy systems that only support TAXII 1.x
- Working with STIX 1.x XML content
- Maintaining compatibility with older threat intelligence feeds

**Use TAXII 2.x when:**
- Building new integrations (preferred for new projects)
- Working with STIX 2.x JSON content
- Using modern threat intelligence platforms

## Getting Started

- [Quick Start](./quickstart.md) - Installation and basic setup
- [TAXII 1.x Guide](./taxii1/overview.md) - Configure and use TAXII 1.x services
- [TAXII 2.x Guide](./taxii2/overview.md) - Configure and use TAXII 2.x API
- [Migration Guide](./migration.md) - Migrate from Python OpenTAXII
