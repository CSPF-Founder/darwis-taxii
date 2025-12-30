//! In-memory DataStore implementation.

use super::{DataSink, DataSource, DataStore, Filter};
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory STIX object store.
///
/// This store keeps all objects in memory and provides fast access
/// by ID. It supports versioning by keeping multiple versions of
/// objects with the same ID.
///
/// The store uses `Arc<RwLock<...>>` internally, so cloning a `MemoryStore`
/// creates a handle to the same underlying data, not a deep copy.
#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    /// Objects indexed by ID and modified timestamp.
    objects: Arc<RwLock<HashMap<String, Vec<StixObject>>>>,
}

impl MemoryStore {
    /// Create a new empty memory store.
    pub fn new() -> Self {
        Self {
            objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a memory store from a list of objects.
    pub fn from_objects(objects: Vec<StixObject>) -> Result<Self> {
        let mut store = Self::new();
        store.add_all(objects)?;
        Ok(store)
    }

    /// Get the number of unique objects (by ID) in the store.
    pub fn len(&self) -> Result<usize> {
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::len"))?;
        Ok(guard.len())
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> Result<bool> {
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::is_empty"))?;
        Ok(guard.is_empty())
    }

    /// Get the total number of object versions in the store.
    pub fn version_count(&self) -> Result<usize> {
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::version_count"))?;
        Ok(guard.values().map(|v| v.len()).sum())
    }

    /// Check if the store contains an object with the given ID.
    pub fn contains(&self, id: &Identifier) -> Result<bool> {
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::contains"))?;
        Ok(guard.contains_key(&id.to_string()))
    }
}

impl DataSource for MemoryStore {
    fn get(&self, id: &Identifier) -> Result<Option<StixObject>> {
        let key = id.to_string();
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::get"))?;
        Ok(guard.get(&key).and_then(|versions| {
            // Return the most recent version
            versions.last().cloned()
        }))
    }

    fn all_versions(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let key = id.to_string();
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::all_versions"))?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }

    fn query(&self, filters: &[Filter]) -> Result<Vec<StixObject>> {
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::query"))?;
        let mut results = Vec::new();

        for versions in guard.values() {
            if let Some(obj) = versions.last() {
                // Convert to JSON for filter matching
                if let Ok(json) = serde_json::to_value(obj) {
                    let matches = filters.iter().all(|f| f.matches(&json));
                    if matches {
                        results.push(obj.clone());
                    }
                }
            }
        }

        Ok(results)
    }

    fn get_all(&self) -> Result<Vec<StixObject>> {
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::get_all"))?;
        let mut results = Vec::new();

        for versions in guard.values() {
            if let Some(obj) = versions.last() {
                results.push(obj.clone());
            }
        }

        Ok(results)
    }
}

impl DataSink for MemoryStore {
    fn add(&mut self, object: StixObject) -> Result<()> {
        let key = object.id().to_string();
        let mut guard = self
            .objects
            .write()
            .map_err(|_| Error::write_lock("MemoryStore::add"))?;

        guard.entry(key).or_insert_with(Vec::new).push(object);

        Ok(())
    }

    fn remove(&mut self, id: &Identifier) -> Result<Option<StixObject>> {
        let key = id.to_string();
        let mut guard = self
            .objects
            .write()
            .map_err(|_| Error::write_lock("MemoryStore::remove"))?;
        Ok(guard.remove(&key).and_then(|mut v| v.pop()))
    }

    fn clear(&mut self) -> Result<()> {
        let mut guard = self
            .objects
            .write()
            .map_err(|_| Error::write_lock("MemoryStore::clear"))?;
        guard.clear();
        Ok(())
    }
}

impl DataStore for MemoryStore {}

/// An owning iterator over objects in a MemoryStore.
pub struct MemoryStoreIterator {
    objects: Vec<StixObject>,
    index: usize,
}

impl Iterator for MemoryStoreIterator {
    type Item = StixObject;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.objects.len() {
            let obj = self.objects[self.index].clone();
            self.index += 1;
            Some(obj)
        } else {
            None
        }
    }
}

impl MemoryStore {
    /// Try to iterate over all objects in the store.
    ///
    /// Returns an error if the lock cannot be acquired.
    pub fn try_iter(&self) -> Result<MemoryStoreIterator> {
        let guard = self
            .objects
            .read()
            .map_err(|_| Error::read_lock("MemoryStore::try_iter"))?;

        let objects = guard
            .values()
            .filter_map(|versions| versions.last().cloned())
            .collect();

        Ok(MemoryStoreIterator { objects, index: 0 })
    }
}

impl IntoIterator for MemoryStore {
    type Item = StixObject;
    type IntoIter = MemoryStoreIterator;

    /// Convert the store into an iterator.
    ///
    /// If the internal lock is poisoned (another thread panicked while holding it),
    /// this method recovers the data and continues. Use `try_iter` for explicit
    /// error handling if you need to detect lock poisoning.
    fn into_iter(self) -> Self::IntoIter {
        // Use unwrap_or_else to recover from poisoned locks - we still get access
        // to the data even if another thread panicked while holding the lock.
        let guard = self.objects.read().unwrap_or_else(|e| e.into_inner());

        let objects = guard
            .values()
            .filter_map(|versions| versions.last().cloned())
            .collect();

        MemoryStoreIterator { objects, index: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::Indicator;
    use crate::vocab::PatternType;

    #[test]
    fn test_memory_store_add_get() {
        let mut store = MemoryStore::new();

        let indicator = Indicator::builder()
            .name("Test Indicator")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let id = indicator.id.clone();
        store.add(StixObject::Indicator(indicator)).unwrap();

        let retrieved = store.get(&id).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_memory_store_query() {
        let mut store = MemoryStore::new();

        let indicator = Indicator::builder()
            .name("Test Indicator")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        store.add(StixObject::Indicator(indicator)).unwrap();

        let filters = vec![Filter::by_type("indicator")];
        let results = store.query(&filters).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_store_remove() {
        let mut store = MemoryStore::new();

        let indicator = Indicator::builder()
            .name("Test Indicator")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let id = indicator.id.clone();
        store.add(StixObject::Indicator(indicator)).unwrap();

        assert!(store.contains(&id).unwrap());
        store.remove(&id).unwrap();
        assert!(!store.contains(&id).unwrap());
    }

    #[test]
    fn test_memory_store_len_and_empty() {
        let mut store = MemoryStore::new();
        assert!(store.is_empty().unwrap());
        assert_eq!(store.len().unwrap(), 0);

        let indicator = Indicator::builder()
            .name("Test Indicator")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        store.add(StixObject::Indicator(indicator)).unwrap();

        assert!(!store.is_empty().unwrap());
        assert_eq!(store.len().unwrap(), 1);
    }
}
