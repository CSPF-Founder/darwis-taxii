//! STIX Environment API
//!
//! This module provides the Environment and ObjectFactory abstractions
//! for working with STIX objects with common defaults.

use chrono::{DateTime, Utc};

use crate::core::bundle::Bundle;
use crate::core::error::{Error, Result};
use crate::core::external_reference::ExternalReference;
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;
use crate::datastore::{CompositeDataSource, DataSink, DataSource, Filter, MemoryStore};
use crate::equivalence::{object_equivalence, object_similarity};
use crate::graph::{StixGraph, graph_similarity};

/// Factory for creating STIX objects with default values.
///
/// # Example
///
/// ```rust,ignore
/// use stix2::environment::ObjectFactory;
/// use stix2::objects::Indicator;
///
/// let factory = ObjectFactory::new()
///     .with_created_by_ref("identity--12345678-1234-1234-1234-123456789012".parse().unwrap());
///
/// // All objects created will have this creator
/// ```
#[derive(Debug, Clone, Default)]
pub struct ObjectFactory {
    /// Default created_by_ref
    created_by_ref: Option<Identifier>,
    /// Default created timestamp
    created: Option<DateTime<Utc>>,
    /// Default external references
    external_references: Option<Vec<ExternalReference>>,
    /// Default object marking refs
    object_marking_refs: Option<Vec<Identifier>>,
    /// Whether to append to list properties or replace them
    list_append: bool,
}

impl ObjectFactory {
    /// Create a new ObjectFactory with no defaults.
    pub fn new() -> Self {
        Self {
            list_append: true,
            ..Default::default()
        }
    }

    /// Set the default creator reference.
    pub fn with_created_by_ref(mut self, creator: Identifier) -> Self {
        self.created_by_ref = Some(creator);
        self
    }

    /// Set the default created timestamp.
    pub fn with_created(mut self, created: DateTime<Utc>) -> Self {
        self.created = Some(created);
        self
    }

    /// Set default external references.
    pub fn with_external_references(mut self, refs: Vec<ExternalReference>) -> Self {
        self.external_references = Some(refs);
        self
    }

    /// Set default object marking references.
    pub fn with_object_marking_refs(mut self, refs: Vec<Identifier>) -> Self {
        self.object_marking_refs = Some(refs);
        self
    }

    /// Set whether to append to list properties (true) or replace them (false).
    pub fn with_list_append(mut self, append: bool) -> Self {
        self.list_append = append;
        self
    }

    /// Get the default created_by_ref.
    pub fn created_by_ref(&self) -> Option<&Identifier> {
        self.created_by_ref.as_ref()
    }

    /// Get the default created timestamp.
    pub fn created(&self) -> Option<DateTime<Utc>> {
        self.created
    }

    /// Get default external references.
    pub fn external_references(&self) -> Option<&Vec<ExternalReference>> {
        self.external_references.as_ref()
    }

    /// Get default object marking references.
    pub fn object_marking_refs(&self) -> Option<&Vec<Identifier>> {
        self.object_marking_refs.as_ref()
    }

    /// Set the default creator (mutable).
    pub fn set_default_creator(&mut self, creator: Option<Identifier>) {
        self.created_by_ref = creator;
    }

    /// Set the default created timestamp (mutable).
    pub fn set_default_created(&mut self, created: Option<DateTime<Utc>>) {
        self.created = created;
    }

    /// Set default external references (mutable).
    pub fn set_default_external_refs(&mut self, refs: Option<Vec<ExternalReference>>) {
        self.external_references = refs;
    }

    /// Set default object marking references (mutable).
    pub fn set_default_object_marking_refs(&mut self, refs: Option<Vec<Identifier>>) {
        self.object_marking_refs = refs;
    }
}

/// STIX Environment for managing objects and data sources.
///
/// The Environment provides a unified API for:
/// - Creating objects with default properties via ObjectFactory
/// - Querying and storing objects via DataStore
/// - Comparing objects for equivalence
///
/// # Example
///
/// ```rust,ignore
/// use stix2::environment::Environment;
/// use stix2::datastore::MemoryStore;
///
/// let store = MemoryStore::new();
/// let env = Environment::new().with_store(store);
///
/// // Query objects
/// let indicators = env.query(&[Filter::new("type", "=", "indicator")]).unwrap();
/// ```
pub struct Environment {
    /// Object factory with defaults
    factory: ObjectFactory,
    /// Composite data source for querying
    source: CompositeDataSource,
    /// Data sink for storing
    sink: Option<Box<dyn DataSinkWrapper>>,
}

// Wrapper trait to allow storing any DataSink
trait DataSinkWrapper: Send + Sync {
    fn add(&mut self, object: StixObject) -> Result<()>;
    fn add_all(&mut self, objects: Vec<StixObject>) -> Result<()>;
    /// Remove an object by ID (reserved for future use).
    #[allow(dead_code)]
    fn remove(&mut self, id: &Identifier) -> Result<Option<StixObject>>;
    fn clear(&mut self) -> Result<()>;
}

impl<T: DataSink + Send + Sync> DataSinkWrapper for T {
    fn add(&mut self, object: StixObject) -> Result<()> {
        DataSink::add(self, object)
    }

    fn add_all(&mut self, objects: Vec<StixObject>) -> Result<()> {
        DataSink::add_all(self, objects)
    }

    fn remove(&mut self, id: &Identifier) -> Result<Option<StixObject>> {
        DataSink::remove(self, id)
    }

    fn clear(&mut self) -> Result<()> {
        DataSink::clear(self)
    }
}

impl Environment {
    /// Create a new Environment.
    pub fn new() -> Self {
        Self {
            factory: ObjectFactory::new(),
            source: CompositeDataSource::new(),
            sink: None,
        }
    }

    /// Create with a custom factory.
    pub fn with_factory(mut self, factory: ObjectFactory) -> Self {
        self.factory = factory;
        self
    }

    /// Add a data source.
    pub fn with_source<S: DataSource + Send + Sync + 'static>(mut self, source: S) -> Self {
        self.source.add_data_source(source);
        self
    }

    /// Set the data sink.
    pub fn with_sink<S: DataSink + Send + Sync + 'static>(mut self, sink: S) -> Self {
        self.sink = Some(Box::new(sink));
        self
    }

    /// Set up with a DataStore (provides both source and sink).
    pub fn with_store(self, store: MemoryStore) -> Self {
        // Clone for source, keep original for sink
        let source_store = store.clone();
        self.with_source(source_store).with_sink(store)
    }

    /// Get the factory.
    pub fn factory(&self) -> &ObjectFactory {
        &self.factory
    }

    /// Get mutable factory.
    pub fn factory_mut(&mut self) -> &mut ObjectFactory {
        &mut self.factory
    }

    // DataSource methods

    /// Get an object by ID.
    pub fn get(&self, id: &Identifier) -> Result<Option<StixObject>> {
        self.source.get(id)
    }

    /// Get all versions of an object.
    pub fn all_versions(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        self.source.all_versions(id)
    }

    /// Query objects with filters.
    pub fn query(&self, filters: &[Filter]) -> Result<Vec<StixObject>> {
        self.source.query(filters)
    }

    /// Get all objects.
    pub fn get_all(&self) -> Result<Vec<StixObject>> {
        self.source.get_all()
    }

    // DataSink methods

    /// Add an object to the sink.
    pub fn add(&mut self, object: StixObject) -> Result<()> {
        if let Some(ref mut sink) = self.sink {
            sink.add(object)
        } else {
            Err(Error::Custom("No data sink configured".to_string()))
        }
    }

    /// Save an object (alias for add).
    pub fn save(&mut self, object: StixObject) -> Result<()> {
        self.add(object)
    }

    /// Add multiple objects.
    pub fn add_all(&mut self, objects: Vec<StixObject>) -> Result<()> {
        if let Some(ref mut sink) = self.sink {
            sink.add_all(objects)
        } else {
            Err(Error::Custom("No data sink configured".to_string()))
        }
    }

    /// Clear all objects from the sink.
    pub fn clear(&mut self) -> Result<()> {
        if let Some(ref mut sink) = self.sink {
            sink.clear()
        } else {
            Err(Error::Custom("No data sink configured".to_string()))
        }
    }

    // Relationship methods

    /// Get relationships where this object is the source.
    pub fn relationships_from(&self, source_id: &Identifier) -> Result<Vec<StixObject>> {
        let filters = vec![
            Filter::new(
                "type",
                crate::datastore::FilterOperator::Equal,
                "relationship",
            ),
            Filter::new(
                "source_ref",
                crate::datastore::FilterOperator::Equal,
                source_id.to_string(),
            ),
        ];
        self.query(&filters)
    }

    /// Get relationships where this object is the target.
    pub fn relationships_to(&self, target_id: &Identifier) -> Result<Vec<StixObject>> {
        let filters = vec![
            Filter::new(
                "type",
                crate::datastore::FilterOperator::Equal,
                "relationship",
            ),
            Filter::new(
                "target_ref",
                crate::datastore::FilterOperator::Equal,
                target_id.to_string(),
            ),
        ];
        self.query(&filters)
    }

    /// Get all relationships involving this object.
    pub fn relationships(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let mut results = self.relationships_from(id)?;
        results.extend(self.relationships_to(id)?);
        Ok(results)
    }

    /// Get objects related to the given object.
    pub fn related_to(&self, id: &Identifier) -> Result<Vec<StixObject>> {
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

    /// Get the creator of an object.
    ///
    /// Note: This requires knowing the creator_by_ref of the object,
    /// which is type-specific. For SDOs, this is available in the common properties.
    pub fn creator_of(&self, creator_ref: &Identifier) -> Result<Option<StixObject>> {
        self.get(creator_ref)
    }

    // Equivalence methods

    /// Calculate similarity between two objects.
    pub fn object_similarity(obj1: &StixObject, obj2: &StixObject) -> f64 {
        object_similarity(obj1, obj2)
    }

    /// Check if two objects are equivalent.
    pub fn object_equivalence(
        obj1: &StixObject,
        obj2: &StixObject,
        threshold: Option<f64>,
    ) -> bool {
        object_equivalence(obj1, obj2, threshold)
    }

    /// Calculate similarity between two graphs.
    pub fn graph_similarity(graph1: &StixGraph, graph2: &StixGraph) -> f64 {
        graph_similarity(graph1, graph2)
    }

    /// Check if two graphs are equivalent.
    pub fn graph_equivalence(
        graph1: &StixGraph,
        graph2: &StixGraph,
        threshold: Option<f64>,
    ) -> bool {
        crate::graph::graphs_equivalent(graph1, graph2, threshold)
    }

    // Parsing

    /// Parse a STIX JSON string.
    pub fn parse(json: &str) -> Result<StixObject> {
        crate::parse(json)
    }

    /// Parse a STIX bundle.
    pub fn parse_bundle(json: &str) -> Result<Bundle> {
        crate::parse_bundle(json)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_factory_defaults() {
        let factory = ObjectFactory::new().with_list_append(true);

        assert!(factory.created_by_ref().is_none());
        assert!(factory.created().is_none());
    }

    #[test]
    fn test_object_factory_with_creator() {
        let creator_id: Identifier = "identity--12345678-1234-1234-1234-123456789012"
            .parse()
            .unwrap();
        let factory = ObjectFactory::new().with_created_by_ref(creator_id.clone());

        assert_eq!(factory.created_by_ref(), Some(&creator_id));
    }

    #[test]
    fn test_environment_creation() {
        let env = Environment::new();
        assert!(env.factory().created_by_ref().is_none());
    }

    #[test]
    fn test_environment_with_store() {
        let store = MemoryStore::new();
        let env = Environment::new().with_store(store);

        // Should be able to query (empty result)
        let results = env.get_all().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_environment_parse() {
        let json = r#"{
            "type": "indicator",
            "spec_version": "2.1",
            "id": "indicator--12345678-1234-1234-1234-123456789012",
            "created": "2023-01-01T00:00:00.000Z",
            "modified": "2023-01-01T00:00:00.000Z",
            "pattern": "[file:name = 'test.exe']",
            "pattern_type": "stix",
            "valid_from": "2023-01-01T00:00:00.000Z"
        }"#;

        let obj = Environment::parse(json).unwrap();
        assert_eq!(obj.type_name(), "indicator");
    }
}
