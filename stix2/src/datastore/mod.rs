//! DataStore Abstractions
//!
//! This module provides abstractions for storing and retrieving STIX objects.

mod composite;
mod filesystem;
mod filter;
mod memory;

#[cfg(feature = "taxii")]
pub mod taxii;

pub use composite::CompositeDataSource;
pub use filesystem::{FileSystemSink, FileSystemSource, FileSystemStore};
pub use filter::{Filter, FilterOperator, FilterValue};
pub use memory::MemoryStore;

#[cfg(feature = "taxii")]
pub use taxii::{TaxiiClient, TaxiiCollectionStore};

use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;

/// Trait for reading STIX objects from a data source.
pub trait DataSource {
    /// Get an object by ID.
    fn get(&self, id: &Identifier) -> Result<Option<StixObject>>;

    /// Get all versions of an object.
    fn all_versions(&self, id: &Identifier) -> Result<Vec<StixObject>>;

    /// Query objects with filters.
    fn query(&self, filters: &[Filter]) -> Result<Vec<StixObject>>;

    /// Get all objects in the data source.
    fn get_all(&self) -> Result<Vec<StixObject>>;
}

/// Trait for writing STIX objects to a data sink.
pub trait DataSink {
    /// Add an object to the store.
    fn add(&mut self, object: StixObject) -> Result<()>;

    /// Add multiple objects to the store.
    fn add_all(&mut self, objects: Vec<StixObject>) -> Result<()> {
        for obj in objects {
            self.add(obj)?;
        }
        Ok(())
    }

    /// Remove an object by ID.
    fn remove(&mut self, id: &Identifier) -> Result<Option<StixObject>>;

    /// Clear all objects from the store.
    fn clear(&mut self) -> Result<()>;
}

/// A combined data source and sink.
pub trait DataStore: DataSource + DataSink {
    /// Get relationships where this object is the source.
    fn relationships_from(&self, source_id: &Identifier) -> Result<Vec<StixObject>> {
        let filters = vec![
            Filter::new("type", FilterOperator::Equal, "relationship"),
            Filter::new("source_ref", FilterOperator::Equal, source_id),
        ];
        self.query(&filters)
    }

    /// Get relationships where this object is the target.
    fn relationships_to(&self, target_id: &Identifier) -> Result<Vec<StixObject>> {
        let filters = vec![
            Filter::new("type", FilterOperator::Equal, "relationship"),
            Filter::new("target_ref", FilterOperator::Equal, target_id),
        ];
        self.query(&filters)
    }

    /// Get all relationships involving this object.
    fn relationships(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let mut results = self.relationships_from(id)?;
        results.extend(self.relationships_to(id)?);
        Ok(results)
    }

    /// Get objects related to the given object.
    fn related_to(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let relationships = self.relationships(id)?;
        let mut related = Vec::new();

        for rel in relationships {
            if let StixObject::Relationship(r) = rel {
                let related_id = if &r.source_ref == id {
                    &r.target_ref
                } else {
                    &r.source_ref
                };

                if let Ok(Some(obj)) = self.get(related_id) {
                    related.push(obj);
                }
            }
        }

        Ok(related)
    }

    /// Get sightings of an object.
    fn sightings_of(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let filters = vec![
            Filter::new("type", FilterOperator::Equal, "sighting"),
            Filter::new("sighting_of_ref", FilterOperator::Equal, id),
        ];
        self.query(&filters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_basic() {
        let store = MemoryStore::new();
        assert!(store.get_all().unwrap().is_empty());
    }
}
