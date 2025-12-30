# stix2

A Rust implementation of STIX 2.1 (Structured Threat Information Expression) for representing and exchanging cyber threat intelligence.

## Features

- **STIX Domain Objects (SDOs)**: Attack Pattern, Campaign, Course of Action, Grouping, Identity, Incident, Indicator, Infrastructure, Intrusion Set, Location, Malware, Malware Analysis, Note, Observed Data, Opinion, Report, Threat Actor, Tool, Vulnerability
- **STIX Relationship Objects (SROs)**: Relationship, Sighting
- **STIX Cyber Observable Objects (SCOs)**: Artifact, Autonomous System, Directory, Domain Name, Email Address, Email Message, File, IPv4/IPv6 Address, MAC Address, Mutex, Network Traffic, Process, Software, URL, User Account, Windows Registry Key, X.509 Certificate
- **Data Markings**: TLP (Traffic Light Protocol), Statement markings
- **Pattern Language**: Full parser for STIX indicator patterns
- **DataStore Abstractions**: Memory store, FileSystem store, Composite data sources
- **Validation**: Property validation per STIX specification
- **Versioning**: Object versioning and revocation utilities
- **Equivalence**: Semantic equivalence and similarity checking
- **Graph Analysis**: Relationship graph traversal and analysis
- **Canonicalization**: Deterministic JSON canonicalization with hashing
- **STIX 2.0 Compatibility**: Parse and detect STIX 2.0 content

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stix2 = "0.1"
```

### Feature Flags

- `default` - Core functionality (no async)
- `async` - Enables async datastore operations with tokio and reqwest
- `taxii` - Enables TAXII client support (includes `async`)

```toml
# With async support
stix2 = { version = "0.1", features = ["async"] }
```

## Quick Start

```rust
use stix2::prelude::*;

fn main() -> stix2::Result<()> {
    // Create an indicator
    let indicator = Indicator::builder()
        .name("Malicious File Hash")
        .pattern("[file:hashes.'SHA-256' = 'abc123']")
        .pattern_type(PatternType::Stix)
        .valid_from_now()
        .build()?;

    // Serialize to JSON
    let json = stix2::serialize_pretty(&indicator)?;
    println!("{}", json);

    // Parse from JSON
    let parsed: StixObject = stix2::parse(&json)?;

    Ok(())
}
```

## Working with Bundles

```rust
use stix2::prelude::*;

fn main() -> stix2::Result<()> {
    // Create a bundle with multiple objects
    let mut bundle = Bundle::new();

    let threat_actor = ThreatActor::builder()
        .name("APT28")
        .threat_actor_types(vec![ThreatActorType::NationState])
        .build()?;

    let malware = Malware::builder()
        .name("X-Agent")
        .is_family(true)
        .malware_types(vec![MalwareType::Backdoor])
        .build()?;

    bundle.add_object(threat_actor);
    bundle.add_object(malware);

    // Serialize the bundle
    let json = stix2::serialize_pretty(&bundle)?;

    Ok(())
}
```

## DataStore Usage

```rust
use stix2::prelude::*;

fn main() -> stix2::Result<()> {
    // Create an in-memory store
    let mut store = MemoryStore::new();

    let indicator = Indicator::builder()
        .name("Test Indicator")
        .pattern("[domain-name:value = 'malicious.com']")
        .pattern_type(PatternType::Stix)
        .valid_from_now()
        .build()?;

    // Add to store
    store.add(indicator.into())?;

    // Query with filters
    let results = store.query(vec![
        Filter::new("type", "=", "indicator"),
    ])?;

    Ok(())
}
```

## STIX Pattern Parsing

```rust
use stix2::patterns::Pattern;

fn main() -> stix2::Result<()> {
    let pattern_str = "[file:hashes.'SHA-256' = 'abc123'] AND [domain-name:value = 'evil.com']";
    let pattern = Pattern::parse(pattern_str)?;

    // Analyze the parsed pattern
    println!("Pattern: {:?}", pattern);

    Ok(())
}
```

## License

BSD-3-Clause

## Links

- [STIX 2.1 Specification](https://docs.oasis-open.org/cti/stix/v2.1/stix-v2.1.html)
