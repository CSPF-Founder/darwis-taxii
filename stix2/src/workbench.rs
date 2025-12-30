//! STIX Workbench API
//!
//! This module provides high-level convenience functions for working with
//! STIX content. It wraps the Environment API with simple function calls.

use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::core::bundle::Bundle;
use crate::core::error::{Error, Result};
use crate::core::external_reference::ExternalReference;
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;
use crate::datastore::{Filter, FilterOperator, MemoryStore};
use crate::environment::Environment;
use crate::objects::{
    AttackPattern, Campaign, CourseOfAction, Grouping, Identity, Incident, Indicator,
    Infrastructure, IntrusionSet, Location, Malware, MalwareAnalysis, Note, ObservedData, Opinion,
    Report, ThreatActor, Tool, Vulnerability,
};
use crate::relationship::{Relationship, Sighting};

// Global workbench environment
static WORKBENCH: Lazy<RwLock<Workbench>> = Lazy::new(|| RwLock::new(Workbench::new()));

/// Internal workbench state
struct Workbench {
    env: Environment,
}

impl Workbench {
    fn new() -> Self {
        let store = MemoryStore::new();
        let env = Environment::new().with_store(store);
        Self { env }
    }
}

// Configuration functions

/// Set the default creator for all objects created via the workbench.
pub fn set_default_creator(creator_ref: Identifier) -> Result<()> {
    let mut wb = WORKBENCH
        .write()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.factory_mut().set_default_creator(Some(creator_ref));
    Ok(())
}

/// Set the default created timestamp for all objects.
pub fn set_default_created(created: chrono::DateTime<chrono::Utc>) -> Result<()> {
    let mut wb = WORKBENCH
        .write()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.factory_mut().set_default_created(Some(created));
    Ok(())
}

/// Set default external references for all objects.
pub fn set_default_external_refs(refs: Vec<ExternalReference>) -> Result<()> {
    let mut wb = WORKBENCH
        .write()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.factory_mut().set_default_external_refs(Some(refs));
    Ok(())
}

/// Set default object marking references for all objects.
pub fn set_default_object_marking_refs(refs: Vec<Identifier>) -> Result<()> {
    let mut wb = WORKBENCH
        .write()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env
        .factory_mut()
        .set_default_object_marking_refs(Some(refs));
    Ok(())
}

// Data access functions

/// Get an object by ID.
pub fn get(id: &Identifier) -> Result<Option<StixObject>> {
    let wb = WORKBENCH
        .read()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.get(id)
}

/// Get all versions of an object.
pub fn all_versions(id: &Identifier) -> Result<Vec<StixObject>> {
    let wb = WORKBENCH
        .read()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.all_versions(id)
}

/// Query objects with filters.
pub fn query(filters: &[Filter]) -> Result<Vec<StixObject>> {
    let wb = WORKBENCH
        .read()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.query(filters)
}

/// Save an object to the workbench.
pub fn save(object: StixObject) -> Result<()> {
    let mut wb = WORKBENCH
        .write()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.add(object)
}

/// Parse a STIX JSON string.
pub fn parse(json: &str) -> Result<StixObject> {
    crate::parse(json)
}

/// Parse a STIX bundle.
pub fn parse_bundle(json: &str) -> Result<Bundle> {
    crate::parse_bundle(json)
}

// Relationship functions

/// Get relationships involving an object.
pub fn relationships(id: &Identifier) -> Result<Vec<StixObject>> {
    let wb = WORKBENCH
        .read()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.relationships(id)
}

/// Get objects related to the given object.
pub fn related_to(id: &Identifier) -> Result<Vec<StixObject>> {
    let wb = WORKBENCH
        .read()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.related_to(id)
}

/// Get an identity by its ID (useful for finding creators).
pub fn get_identity(id: &Identifier) -> Result<Option<StixObject>> {
    let wb = WORKBENCH
        .read()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.get(id)
}

// Object type query functions

fn query_by_type(type_name: &str) -> Result<Vec<StixObject>> {
    let filters = vec![Filter::new("type", FilterOperator::Equal, type_name)];
    query(&filters)
}

/// Get all attack patterns.
pub fn attack_patterns() -> Result<Vec<AttackPattern>> {
    Ok(query_by_type("attack-pattern")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::AttackPattern(ap) = obj {
                Some(ap)
            } else {
                None
            }
        })
        .collect())
}

/// Get all campaigns.
pub fn campaigns() -> Result<Vec<Campaign>> {
    Ok(query_by_type("campaign")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Campaign(c) = obj {
                Some(c)
            } else {
                None
            }
        })
        .collect())
}

/// Get all courses of action.
pub fn courses_of_action() -> Result<Vec<CourseOfAction>> {
    Ok(query_by_type("course-of-action")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::CourseOfAction(coa) = obj {
                Some(coa)
            } else {
                None
            }
        })
        .collect())
}

/// Get all groupings.
pub fn groupings() -> Result<Vec<Grouping>> {
    Ok(query_by_type("grouping")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Grouping(g) = obj {
                Some(g)
            } else {
                None
            }
        })
        .collect())
}

/// Get all identities.
pub fn identities() -> Result<Vec<Identity>> {
    Ok(query_by_type("identity")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Identity(i) = obj {
                Some(i)
            } else {
                None
            }
        })
        .collect())
}

/// Get all incidents.
pub fn incidents() -> Result<Vec<Incident>> {
    Ok(query_by_type("incident")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Incident(i) = obj {
                Some(i)
            } else {
                None
            }
        })
        .collect())
}

/// Get all indicators.
pub fn indicators() -> Result<Vec<Indicator>> {
    Ok(query_by_type("indicator")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Indicator(i) = obj {
                Some(i)
            } else {
                None
            }
        })
        .collect())
}

/// Get all infrastructure.
pub fn infrastructures() -> Result<Vec<Infrastructure>> {
    Ok(query_by_type("infrastructure")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Infrastructure(i) = obj {
                Some(i)
            } else {
                None
            }
        })
        .collect())
}

/// Get all intrusion sets.
pub fn intrusion_sets() -> Result<Vec<IntrusionSet>> {
    Ok(query_by_type("intrusion-set")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::IntrusionSet(is) = obj {
                Some(is)
            } else {
                None
            }
        })
        .collect())
}

/// Get all locations.
pub fn locations() -> Result<Vec<Location>> {
    Ok(query_by_type("location")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Location(l) = obj {
                Some(l)
            } else {
                None
            }
        })
        .collect())
}

/// Get all malware.
pub fn malware() -> Result<Vec<Malware>> {
    Ok(query_by_type("malware")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Malware(m) = obj {
                Some(m)
            } else {
                None
            }
        })
        .collect())
}

/// Get all malware analyses.
pub fn malware_analyses() -> Result<Vec<MalwareAnalysis>> {
    Ok(query_by_type("malware-analysis")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::MalwareAnalysis(ma) = obj {
                Some(ma)
            } else {
                None
            }
        })
        .collect())
}

/// Get all notes.
pub fn notes() -> Result<Vec<Note>> {
    Ok(query_by_type("note")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Note(n) = obj {
                Some(n)
            } else {
                None
            }
        })
        .collect())
}

/// Get all observed data.
pub fn observed_data() -> Result<Vec<ObservedData>> {
    Ok(query_by_type("observed-data")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::ObservedData(od) = obj {
                Some(od)
            } else {
                None
            }
        })
        .collect())
}

/// Get all opinions.
pub fn opinions() -> Result<Vec<Opinion>> {
    Ok(query_by_type("opinion")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Opinion(o) = obj {
                Some(o)
            } else {
                None
            }
        })
        .collect())
}

/// Get all reports.
pub fn reports() -> Result<Vec<Report>> {
    Ok(query_by_type("report")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Report(r) = obj {
                Some(r)
            } else {
                None
            }
        })
        .collect())
}

/// Get all threat actors.
pub fn threat_actors() -> Result<Vec<ThreatActor>> {
    Ok(query_by_type("threat-actor")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::ThreatActor(ta) = obj {
                Some(ta)
            } else {
                None
            }
        })
        .collect())
}

/// Get all tools.
pub fn tools() -> Result<Vec<Tool>> {
    Ok(query_by_type("tool")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Tool(t) = obj {
                Some(t)
            } else {
                None
            }
        })
        .collect())
}

/// Get all vulnerabilities.
pub fn vulnerabilities() -> Result<Vec<Vulnerability>> {
    Ok(query_by_type("vulnerability")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Vulnerability(v) = obj {
                Some(v)
            } else {
                None
            }
        })
        .collect())
}

/// Get all relationships.
pub fn all_relationships() -> Result<Vec<Relationship>> {
    Ok(query_by_type("relationship")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Relationship(r) = obj {
                Some(r)
            } else {
                None
            }
        })
        .collect())
}

/// Get all sightings.
pub fn sightings() -> Result<Vec<Sighting>> {
    Ok(query_by_type("sighting")?
        .into_iter()
        .filter_map(|obj| {
            if let StixObject::Sighting(s) = obj {
                Some(s)
            } else {
                None
            }
        })
        .collect())
}

/// Clear all objects from the workbench.
pub fn clear() -> Result<()> {
    let mut wb = WORKBENCH
        .write()
        .map_err(|_| Error::Custom("Failed to acquire workbench lock".to_string()))?;
    wb.env.clear()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::PatternType;
    use std::sync::Mutex;

    // Mutex to ensure workbench tests run serially
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_workbench_save_and_query() {
        let _lock = TEST_MUTEX.lock().unwrap();

        // Clear any existing state
        clear().unwrap();

        // Create and save an indicator
        let indicator = Indicator::builder()
            .name("Test Indicator")
            .pattern("[file:name = 'test.exe']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        save(StixObject::Indicator(indicator)).unwrap();

        // Query indicators
        let results = indicators().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, Some("Test Indicator".to_string()));

        // Clean up
        clear().unwrap();
    }

    #[test]
    fn test_workbench_get() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear().unwrap();

        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[file:name = 'test.exe']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let id = indicator.id.clone();
        save(StixObject::Indicator(indicator)).unwrap();

        let result = get(&id).unwrap();
        assert!(result.is_some());

        clear().unwrap();
    }
}
