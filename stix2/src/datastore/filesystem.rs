//! FileSystem DataStore
//!
//! Provides file-based storage for STIX objects.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;

use super::{DataSink, DataSource, DataStore, Filter};

/// A file system-based store for STIX objects.
///
/// Objects are stored in a directory structure:
/// - `<stix_dir>/<type>/<id>.json` for unversioned objects (SCOs, marking-definitions)
/// - `<stix_dir>/<type>/<id>/<modified>.json` for versioned objects (SDOs, SROs)
#[derive(Debug, Clone)]
pub struct FileSystemStore {
    stix_dir: PathBuf,
    allow_custom: bool,
    bundlify: bool,
}

/// A file system source for reading STIX objects.
#[derive(Debug, Clone)]
pub struct FileSystemSource {
    stix_dir: PathBuf,
    /// Reserved for custom type filtering.
    #[allow(dead_code)]
    allow_custom: bool,
}

/// A file system sink for writing STIX objects.
#[derive(Debug, Clone)]
pub struct FileSystemSink {
    stix_dir: PathBuf,
    /// Reserved for custom type filtering.
    #[allow(dead_code)]
    allow_custom: bool,
    bundlify: bool,
}

impl FileSystemStore {
    /// Create a new FileSystemStore.
    ///
    /// # Arguments
    /// * `stix_dir` - Path to the directory containing STIX objects
    /// * `allow_custom` - Whether to allow custom STIX objects
    /// * `bundlify` - Whether to wrap objects in bundles when saving
    pub fn new(stix_dir: impl AsRef<Path>, allow_custom: bool, bundlify: bool) -> Result<Self> {
        let path = stix_dir.as_ref().to_path_buf();
        if !path.exists() {
            return Err(Error::io(format!(
                "Directory does not exist: {}",
                path.display()
            )));
        }
        Ok(Self {
            stix_dir: path,
            allow_custom,
            bundlify,
        })
    }

    /// Get the path to the STIX directory.
    pub fn stix_dir(&self) -> &Path {
        &self.stix_dir
    }
}

impl FileSystemSource {
    /// Create a new FileSystemSource.
    pub fn new(stix_dir: impl AsRef<Path>, allow_custom: bool) -> Result<Self> {
        let path = stix_dir.as_ref().to_path_buf();
        if !path.exists() {
            return Err(Error::io(format!(
                "Directory does not exist: {}",
                path.display()
            )));
        }
        Ok(Self {
            stix_dir: path,
            allow_custom,
        })
    }

    /// Get the path to the STIX directory.
    pub fn stix_dir(&self) -> &Path {
        &self.stix_dir
    }

    fn read_object_from_file(&self, path: &Path) -> Result<StixObject> {
        let mut file = File::open(path).map_err(|e| Error::io(e.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::io(e.to_string()))?;
        crate::parse(&contents)
    }

    fn is_versioned_type(&self, type_name: &str) -> bool {
        // SCOs and marking-definitions are unversioned
        !matches!(
            type_name,
            "artifact"
                | "autonomous-system"
                | "directory"
                | "domain-name"
                | "email-addr"
                | "email-message"
                | "file"
                | "ipv4-addr"
                | "ipv6-addr"
                | "mac-addr"
                | "mutex"
                | "network-traffic"
                | "process"
                | "software"
                | "url"
                | "user-account"
                | "windows-registry-key"
                | "x509-certificate"
                | "marking-definition"
        )
    }

    fn collect_objects_from_type_dir(
        &self,
        type_path: &Path,
        type_name: &str,
        filters: &[Filter],
    ) -> Result<Vec<StixObject>> {
        let mut results = Vec::new();

        if !type_path.exists() {
            return Ok(results);
        }

        if self.is_versioned_type(type_name) {
            // Versioned: look for id subdirectories
            for entry in fs::read_dir(type_path).map_err(|e| Error::io(e.to_string()))? {
                let entry = entry.map_err(|e| Error::io(e.to_string()))?;
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    // This is an ID directory, look for version files
                    for version_entry in
                        fs::read_dir(&entry_path).map_err(|e| Error::io(e.to_string()))?
                    {
                        let version_entry = version_entry.map_err(|e| Error::io(e.to_string()))?;
                        let version_path = version_entry.path();

                        if version_path.extension().is_some_and(|e| e == "json")
                            && let Ok(obj) = self.read_object_from_file(&version_path)
                            && self.object_matches_filters(&obj, filters)
                        {
                            results.push(obj);
                        }
                    }
                } else if entry_path.extension().is_some_and(|e| e == "json") {
                    // Backward compatibility: plain files in type directory
                    if let Ok(obj) = self.read_object_from_file(&entry_path)
                        && self.object_matches_filters(&obj, filters)
                    {
                        results.push(obj);
                    }
                }
            }
        } else {
            // Unversioned: look for files directly in type directory
            for entry in fs::read_dir(type_path).map_err(|e| Error::io(e.to_string()))? {
                let entry = entry.map_err(|e| Error::io(e.to_string()))?;
                let entry_path = entry.path();

                if entry_path.extension().is_some_and(|e| e == "json")
                    && let Ok(obj) = self.read_object_from_file(&entry_path)
                    && self.object_matches_filters(&obj, filters)
                {
                    results.push(obj);
                }
            }
        }

        Ok(results)
    }

    fn object_matches_filters(&self, obj: &StixObject, filters: &[Filter]) -> bool {
        if filters.is_empty() {
            return true;
        }

        let json_value = match serde_json::to_value(obj) {
            Ok(v) => v,
            Err(_) => return false,
        };

        filters.iter().all(|f| f.matches(&json_value))
    }
}

impl FileSystemSink {
    /// Create a new FileSystemSink.
    pub fn new(stix_dir: impl AsRef<Path>, allow_custom: bool, bundlify: bool) -> Result<Self> {
        let path = stix_dir.as_ref().to_path_buf();
        if !path.exists() {
            return Err(Error::io(format!(
                "Directory does not exist: {}",
                path.display()
            )));
        }
        Ok(Self {
            stix_dir: path,
            allow_custom,
            bundlify,
        })
    }

    /// Get the path to the STIX directory.
    pub fn stix_dir(&self) -> &Path {
        &self.stix_dir
    }

    fn timestamp_to_filename(timestamp: &str) -> String {
        timestamp
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect()
    }

    fn get_type_from_object(obj: &StixObject) -> &str {
        match obj {
            StixObject::AttackPattern(_) => "attack-pattern",
            StixObject::Campaign(_) => "campaign",
            StixObject::CourseOfAction(_) => "course-of-action",
            StixObject::Grouping(_) => "grouping",
            StixObject::Identity(_) => "identity",
            StixObject::Incident(_) => "incident",
            StixObject::Indicator(_) => "indicator",
            StixObject::Infrastructure(_) => "infrastructure",
            StixObject::IntrusionSet(_) => "intrusion-set",
            StixObject::Location(_) => "location",
            StixObject::Malware(_) => "malware",
            StixObject::MalwareAnalysis(_) => "malware-analysis",
            StixObject::Note(_) => "note",
            StixObject::ObservedData(_) => "observed-data",
            StixObject::Opinion(_) => "opinion",
            StixObject::Report(_) => "report",
            StixObject::ThreatActor(_) => "threat-actor",
            StixObject::Tool(_) => "tool",
            StixObject::Vulnerability(_) => "vulnerability",
            StixObject::Relationship(_) => "relationship",
            StixObject::Sighting(_) => "sighting",
            StixObject::MarkingDefinition(_) => "marking-definition",
            StixObject::LanguageContent(_) => "language-content",
            StixObject::Artifact(_) => "artifact",
            StixObject::AutonomousSystem(_) => "autonomous-system",
            StixObject::Directory(_) => "directory",
            StixObject::DomainName(_) => "domain-name",
            StixObject::EmailAddress(_) => "email-addr",
            StixObject::EmailMessage(_) => "email-message",
            StixObject::File(_) => "file",
            StixObject::IPv4Address(_) => "ipv4-addr",
            StixObject::IPv6Address(_) => "ipv6-addr",
            StixObject::MacAddress(_) => "mac-addr",
            StixObject::Mutex(_) => "mutex",
            StixObject::NetworkTraffic(_) => "network-traffic",
            StixObject::Process(_) => "process",
            StixObject::Software(_) => "software",
            StixObject::Url(_) => "url",
            StixObject::UserAccount(_) => "user-account",
            StixObject::WindowsRegistryKey(_) => "windows-registry-key",
            StixObject::X509Certificate(_) => "x509-certificate",
            StixObject::Custom(c) => &c.type_,
        }
    }

    fn is_versioned(obj: &StixObject) -> bool {
        // SDOs and SROs are versioned, SCOs and marking-definitions are not
        matches!(
            obj,
            StixObject::AttackPattern(_)
                | StixObject::Campaign(_)
                | StixObject::CourseOfAction(_)
                | StixObject::Grouping(_)
                | StixObject::Identity(_)
                | StixObject::Incident(_)
                | StixObject::Indicator(_)
                | StixObject::Infrastructure(_)
                | StixObject::IntrusionSet(_)
                | StixObject::Location(_)
                | StixObject::Malware(_)
                | StixObject::MalwareAnalysis(_)
                | StixObject::Note(_)
                | StixObject::ObservedData(_)
                | StixObject::Opinion(_)
                | StixObject::Report(_)
                | StixObject::ThreatActor(_)
                | StixObject::Tool(_)
                | StixObject::Vulnerability(_)
                | StixObject::Relationship(_)
                | StixObject::Sighting(_)
                | StixObject::LanguageContent(_)
        )
    }
}

impl DataSource for FileSystemSource {
    fn get(&self, id: &Identifier) -> Result<Option<StixObject>> {
        let all_versions = self.all_versions(id)?;
        Ok(all_versions.into_iter().max_by(|a, b| {
            let a_modified = get_modified(a);
            let b_modified = get_modified(b);
            a_modified.cmp(&b_modified)
        }))
    }

    fn all_versions(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let type_name = id.object_type();
        let type_path = self.stix_dir.join(type_name);

        if !type_path.exists() {
            return Ok(Vec::new());
        }

        let filters = vec![Filter::eq("id", id.to_string())];
        self.collect_objects_from_type_dir(&type_path, type_name, &filters)
    }

    fn query(&self, filters: &[Filter]) -> Result<Vec<StixObject>> {
        let mut results = Vec::new();

        // Check if we can optimize by type filter
        let type_filter = filters.iter().find(|f| f.property == "type");

        if let Some(tf) = type_filter {
            if let super::filter::FilterValue::String(type_name) = &tf.value {
                let type_path = self.stix_dir.join(type_name);
                results.extend(self.collect_objects_from_type_dir(&type_path, type_name, filters)?);
            }
        } else {
            // No type filter, search all type directories
            if let Ok(entries) = fs::read_dir(&self.stix_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir()
                        && let Some(type_name) = entry_path.file_name().and_then(|n| n.to_str())
                    {
                        results.extend(self.collect_objects_from_type_dir(
                            &entry_path,
                            type_name,
                            filters,
                        )?);
                    }
                }
            }
        }

        Ok(results)
    }

    fn get_all(&self) -> Result<Vec<StixObject>> {
        self.query(&[])
    }
}

impl DataSink for FileSystemSink {
    fn add(&mut self, object: StixObject) -> Result<()> {
        let type_name = Self::get_type_from_object(&object);
        let type_dir = self.stix_dir.join(type_name);

        // Ensure type directory exists
        fs::create_dir_all(&type_dir).map_err(|e| Error::io(e.to_string()))?;

        let id = get_id(&object);
        let file_path = if Self::is_versioned(&object) {
            // Versioned: create id subdirectory and use modified timestamp as filename
            let id_dir = type_dir.join(id.to_string());
            fs::create_dir_all(&id_dir).map_err(|e| Error::io(e.to_string()))?;

            let modified = get_modified(&object).unwrap_or_default();
            let filename = Self::timestamp_to_filename(&modified);
            id_dir.join(format!("{}.json", filename))
        } else {
            // Unversioned: use id as filename
            type_dir.join(format!("{}.json", id))
        };

        // Don't overwrite existing files
        if file_path.exists() {
            return Err(Error::io(format!(
                "File already exists: {}",
                file_path.display()
            )));
        }

        let json = if self.bundlify {
            let bundle = crate::core::bundle::Bundle::from_objects(vec![object]);
            serde_json::to_string_pretty(&bundle)
                .map_err(|e| Error::serialization(e.to_string()))?
        } else {
            serde_json::to_string_pretty(&object)
                .map_err(|e| Error::serialization(e.to_string()))?
        };

        let mut file = File::create(&file_path).map_err(|e| Error::io(e.to_string()))?;
        file.write_all(json.as_bytes())
            .map_err(|e| Error::io(e.to_string()))?;

        Ok(())
    }

    fn remove(&mut self, id: &Identifier) -> Result<Option<StixObject>> {
        let type_name = id.object_type();
        let type_dir = self.stix_dir.join(type_name);

        if !type_dir.exists() {
            return Ok(None);
        }

        // Try versioned path first
        let id_dir = type_dir.join(id.to_string());
        if id_dir.exists() && id_dir.is_dir() {
            // Remove the entire id directory
            fs::remove_dir_all(&id_dir).map_err(|e| Error::io(e.to_string()))?;
            return Ok(None); // We don't return the removed object
        }

        // Try unversioned path
        let file_path = type_dir.join(format!("{}.json", id));
        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| Error::io(e.to_string()))?;
        }

        Ok(None)
    }

    fn clear(&mut self) -> Result<()> {
        // Remove all type directories
        if let Ok(entries) = fs::read_dir(&self.stix_dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    fs::remove_dir_all(&entry_path).map_err(|e| Error::io(e.to_string()))?;
                }
            }
        }
        Ok(())
    }
}

impl DataSource for FileSystemStore {
    fn get(&self, id: &Identifier) -> Result<Option<StixObject>> {
        FileSystemSource::new(&self.stix_dir, self.allow_custom)?.get(id)
    }

    fn all_versions(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        FileSystemSource::new(&self.stix_dir, self.allow_custom)?.all_versions(id)
    }

    fn query(&self, filters: &[Filter]) -> Result<Vec<StixObject>> {
        FileSystemSource::new(&self.stix_dir, self.allow_custom)?.query(filters)
    }

    fn get_all(&self) -> Result<Vec<StixObject>> {
        FileSystemSource::new(&self.stix_dir, self.allow_custom)?.get_all()
    }
}

impl DataSink for FileSystemStore {
    fn add(&mut self, object: StixObject) -> Result<()> {
        FileSystemSink::new(&self.stix_dir, self.allow_custom, self.bundlify)?.add(object)
    }

    fn remove(&mut self, id: &Identifier) -> Result<Option<StixObject>> {
        FileSystemSink::new(&self.stix_dir, self.allow_custom, self.bundlify)?.remove(id)
    }

    fn clear(&mut self) -> Result<()> {
        FileSystemSink::new(&self.stix_dir, self.allow_custom, self.bundlify)?.clear()
    }
}

impl DataStore for FileSystemStore {}

// Helper functions

fn get_id(obj: &StixObject) -> &Identifier {
    match obj {
        StixObject::AttackPattern(o) => &o.id,
        StixObject::Campaign(o) => &o.id,
        StixObject::CourseOfAction(o) => &o.id,
        StixObject::Grouping(o) => &o.id,
        StixObject::Identity(o) => &o.id,
        StixObject::Incident(o) => &o.id,
        StixObject::Indicator(o) => &o.id,
        StixObject::Infrastructure(o) => &o.id,
        StixObject::IntrusionSet(o) => &o.id,
        StixObject::Location(o) => &o.id,
        StixObject::Malware(o) => &o.id,
        StixObject::MalwareAnalysis(o) => &o.id,
        StixObject::Note(o) => &o.id,
        StixObject::ObservedData(o) => &o.id,
        StixObject::Opinion(o) => &o.id,
        StixObject::Report(o) => &o.id,
        StixObject::ThreatActor(o) => &o.id,
        StixObject::Tool(o) => &o.id,
        StixObject::Vulnerability(o) => &o.id,
        StixObject::Relationship(o) => &o.id,
        StixObject::Sighting(o) => &o.id,
        StixObject::MarkingDefinition(o) => &o.id,
        StixObject::LanguageContent(o) => &o.id,
        StixObject::Artifact(o) => &o.id,
        StixObject::AutonomousSystem(o) => &o.id,
        StixObject::Directory(o) => &o.id,
        StixObject::DomainName(o) => &o.id,
        StixObject::EmailAddress(o) => &o.id,
        StixObject::EmailMessage(o) => &o.id,
        StixObject::File(o) => &o.id,
        StixObject::IPv4Address(o) => &o.id,
        StixObject::IPv6Address(o) => &o.id,
        StixObject::MacAddress(o) => &o.id,
        StixObject::Mutex(o) => &o.id,
        StixObject::NetworkTraffic(o) => &o.id,
        StixObject::Process(o) => &o.id,
        StixObject::Software(o) => &o.id,
        StixObject::Url(o) => &o.id,
        StixObject::UserAccount(o) => &o.id,
        StixObject::WindowsRegistryKey(o) => &o.id,
        StixObject::X509Certificate(o) => &o.id,
        StixObject::Custom(o) => &o.id,
    }
}

fn get_modified(obj: &StixObject) -> Option<String> {
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
        _ => None, // SCOs and marking-definitions don't have modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_filesystem_store_creation() {
        let temp_dir = env::temp_dir().join("stix2_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let store = FileSystemStore::new(&temp_dir, true, false);
        assert!(store.is_ok());

        fs::remove_dir_all(&temp_dir).ok();
    }
}
