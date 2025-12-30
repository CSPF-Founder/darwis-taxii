//! Composite DataSource
//!
//! Provides a federated data source that queries multiple data sources.

use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;
use crate::utils::deduplicate;

use super::{DataSource, Filter};

/// A composite data source that federates queries across multiple data sources.
///
/// When a query is made, it is sent to all attached data sources, and the
/// results are combined and deduplicated.
pub struct CompositeDataSource {
    data_sources: Vec<Box<dyn DataSource + Send + Sync>>,
}

impl Default for CompositeDataSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeDataSource {
    /// Create a new empty CompositeDataSource.
    pub fn new() -> Self {
        Self {
            data_sources: Vec::new(),
        }
    }

    /// Add a data source to the composite.
    pub fn add_data_source<D: DataSource + Send + Sync + 'static>(&mut self, source: D) {
        self.data_sources.push(Box::new(source));
    }

    /// Add multiple data sources.
    pub fn add_data_sources<D: DataSource + Send + Sync + 'static>(
        &mut self,
        sources: impl IntoIterator<Item = D>,
    ) {
        for source in sources {
            self.add_data_source(source);
        }
    }

    /// Remove all data sources.
    pub fn clear_data_sources(&mut self) {
        self.data_sources.clear();
    }

    /// Check if the composite has any data sources.
    pub fn has_data_sources(&self) -> bool {
        !self.data_sources.is_empty()
    }

    /// Get the number of attached data sources.
    pub fn data_source_count(&self) -> usize {
        self.data_sources.len()
    }
}

impl DataSource for CompositeDataSource {
    fn get(&self, id: &Identifier) -> Result<Option<StixObject>> {
        let mut all_data = Vec::new();

        for source in &self.data_sources {
            if let Ok(Some(obj)) = source.get(id) {
                all_data.push(obj);
            }
        }

        if all_data.is_empty() {
            return Ok(None);
        }

        // Return the most recent version
        Ok(all_data.into_iter().max_by(|a, b| {
            let a_modified = get_modified_opt(a);
            let b_modified = get_modified_opt(b);
            a_modified.cmp(&b_modified)
        }))
    }

    fn all_versions(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let mut all_data = Vec::new();

        for source in &self.data_sources {
            if let Ok(versions) = source.all_versions(id) {
                all_data.extend(versions);
            }
        }

        Ok(deduplicate(all_data))
    }

    fn query(&self, filters: &[Filter]) -> Result<Vec<StixObject>> {
        let mut all_data = Vec::new();

        for source in &self.data_sources {
            if let Ok(results) = source.query(filters) {
                all_data.extend(results);
            }
        }

        Ok(deduplicate(all_data))
    }

    fn get_all(&self) -> Result<Vec<StixObject>> {
        let mut all_data = Vec::new();

        for source in &self.data_sources {
            if let Ok(objects) = source.get_all() {
                all_data.extend(objects);
            }
        }

        Ok(deduplicate(all_data))
    }
}

fn get_modified_opt(obj: &StixObject) -> Option<String> {
    match obj {
        StixObject::AttackPattern(o) => Some(o.common.modified.to_string()),
        StixObject::Campaign(o) => Some(o.common.modified.to_string()),
        StixObject::CourseOfAction(o) => Some(o.common.modified.to_string()),
        StixObject::Grouping(o) => Some(o.common.modified.to_string()),
        StixObject::Identity(o) => Some(o.common.modified.to_string()),
        StixObject::Incident(o) => Some(o.common.modified.to_string()),
        StixObject::Indicator(o) => Some(o.common.modified.to_string()),
        StixObject::Infrastructure(o) => Some(o.common.modified.to_string()),
        StixObject::IntrusionSet(o) => Some(o.common.modified.to_string()),
        StixObject::Location(o) => Some(o.common.modified.to_string()),
        StixObject::Malware(o) => Some(o.common.modified.to_string()),
        StixObject::MalwareAnalysis(o) => Some(o.common.modified.to_string()),
        StixObject::Note(o) => Some(o.common.modified.to_string()),
        StixObject::ObservedData(o) => Some(o.common.modified.to_string()),
        StixObject::Opinion(o) => Some(o.common.modified.to_string()),
        StixObject::Report(o) => Some(o.common.modified.to_string()),
        StixObject::ThreatActor(o) => Some(o.common.modified.to_string()),
        StixObject::Tool(o) => Some(o.common.modified.to_string()),
        StixObject::Vulnerability(o) => Some(o.common.modified.to_string()),
        StixObject::Relationship(o) => Some(o.common.modified.to_string()),
        StixObject::Sighting(o) => Some(o.common.modified.to_string()),
        StixObject::LanguageContent(o) => Some(o.common.modified.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datastore::MemoryStore;

    #[test]
    fn test_composite_creation() {
        let composite = CompositeDataSource::new();
        assert!(!composite.has_data_sources());
    }

    #[test]
    fn test_add_data_source() {
        let mut composite = CompositeDataSource::new();
        let memory_store = MemoryStore::new();
        composite.add_data_source(memory_store);
        assert!(composite.has_data_sources());
        assert_eq!(composite.data_source_count(), 1);
    }
}
