//! STIX Extensions
//!
//! This module provides extension types for STIX objects, particularly
//! for Cyber Observable Objects (SCOs).

use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::validation::Constrained;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Windows PE Binary extension for File objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowsPeBinaryExt {
    /// The type of PE binary (required). From windows-pebinary-type-ov.
    pub pe_type: String,
    /// The import hash (imphash) of the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imphash: Option<String>,
    /// The machine type of the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_hex: Option<String>,
    /// The number of sections in the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_sections: Option<u32>,
    /// The date/time stamp of the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_date_stamp: Option<Timestamp>,
    /// The pointer to the symbol table of the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_to_symbol_table_hex: Option<String>,
    /// The number of symbols in the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_symbols: Option<u32>,
    /// The size of the optional header of the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_optional_header: Option<u32>,
    /// The characteristics of the PE binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characteristics_hex: Option<String>,
    /// Hashes of the file header (COFF header).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub file_header_hashes: IndexMap<String, String>,
    /// The PE file header's optional header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_header: Option<WindowsPeOptionalHeaderType>,
    /// The sections in the PE binary.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<WindowsPeSection>,
}

/// Windows PE Section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowsPeSection {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entropy: Option<f64>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub hashes: IndexMap<String, String>,
}

/// Archive file extension.
///
/// Contains information about archive files (e.g., ZIP, TAR).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveExt {
    /// References to file objects contained in the archive (required).
    /// Each reference must be a file--<uuid> identifier.
    pub contains_refs: Vec<Identifier>,
    /// A comment included as part of the archive file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// PDF extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_optimized: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_info_dict: Option<IndexMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdfid0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdfid1: Option<String>,
}

/// Raster image extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RasterImageExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits_per_pixel: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exif_tags: Option<IndexMap<String, Value>>,
}

/// NTFS extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NtfsExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternate_data_streams: Vec<AlternateDataStream>,
}

/// NTFS Alternate Data Stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlternateDataStream {
    pub name: String,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub hashes: IndexMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Windows Process extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowsProcessExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aslr_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_sid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_info: Option<IndexMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity_level: Option<String>,
}

/// Windows Service extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowsServiceExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub descriptions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_type: Option<String>,
    /// References to file objects for the service DLLs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub service_dll_refs: Vec<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_status: Option<String>,
}

/// Unix Account extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnixAccountExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
}

/// HTTP Request extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpRequestExt {
    pub request_method: String,
    pub request_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_version: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub request_header: IndexMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_body_length: Option<u64>,
    /// Reference to an artifact object containing the message body data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_body_data_ref: Option<Identifier>,
}

impl Constrained for HttpRequestExt {
    /// Validate HttpRequestExt constraints.
    ///
    /// - `message_body_data_ref` must reference an `artifact` type.
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::check_optional_ref_type;

        check_optional_ref_type(
            self.message_body_data_ref.as_ref(),
            "message_body_data_ref",
            &["artifact"],
        )?;

        Ok(())
    }
}

/// ICMP extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IcmpExt {
    pub icmp_type_hex: String,
    pub icmp_code_hex: String,
}

/// Socket extension for network-traffic objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocketExt {
    /// The address family (required). From network-socket-address-family-enum.
    pub address_family: String,
    /// Whether the socket is in blocking mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_blocking: Option<bool>,
    /// Whether the socket is in listening mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_listening: Option<bool>,
    /// Socket options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<IndexMap<String, Value>>,
    /// The socket type. From network-socket-type-enum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_type: Option<String>,
    /// The socket file descriptor (non-negative integer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_descriptor: Option<u64>,
    /// The Windows socket handle (non-negative integer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_handle: Option<u64>,
}

impl Constrained for SocketExt {
    /// Validate SocketExt constraints.
    ///
    /// - Options keys must start with SO_, ICMP_, ICMP6_, IP_, IPV6_, MCAST_, TCP_, or IRLMP_
    /// - Options values must be integers
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::{check_socket_options_keys, check_socket_options_values};

        if let Some(options) = &self.options {
            let keys: Vec<&str> = options.keys().map(|s| s.as_str()).collect();
            check_socket_options_keys(&keys)?;

            let values: Vec<&Value> = options.values().collect();
            check_socket_options_values(&values)?;
        }
        Ok(())
    }
}

/// TCP extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcpExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_flags_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_flags_hex: Option<String>,
}

/// Email MIME Component for multipart email messages.
///
/// Specifies one component of a multi-part email body.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EmailMimeComponent {
    /// The value of the "Content-Type" header of the MIME part.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// The value of the "Content-Disposition" header of the MIME part.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<String>,
    /// The contents of the MIME part if the content_type is not "text/plain".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Reference to an Artifact or File object containing the decoded body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_raw_ref: Option<Identifier>,
}

impl Constrained for EmailMimeComponent {
    /// Validate EmailMIMEComponent constraints.
    ///
    /// - At least one of `body` or `body_raw_ref` must be present.
    /// - `body_raw_ref` must reference only `artifact` or `file` types.
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::check_optional_ref_type;

        if self.body.is_none() && self.body_raw_ref.is_none() {
            return Err(Error::AtLeastOneRequired(vec![
                "body".to_string(),
                "body_raw_ref".to_string(),
            ]));
        }

        check_optional_ref_type(
            self.body_raw_ref.as_ref(),
            "body_raw_ref",
            &["artifact", "file"],
        )?;

        Ok(())
    }
}

/// X.509 v3 Extensions type.
///
/// Captures properties from X.509 v3 certificate extensions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct X509V3ExtensionsType {
    /// Whether the certificate is a CA certificate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_constraints: Option<String>,
    /// The DNS name binding to the certificate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_constraints: Option<String>,
    /// Extension for defining certificate policies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_constraints: Option<String>,
    /// Key usage extension defines the purpose of the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_usage: Option<String>,
    /// Extended key usage extension further refines key usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_key_usage: Option<String>,
    /// Subject key identifier extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_key_identifier: Option<String>,
    /// Authority key identifier extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_key_identifier: Option<String>,
    /// Subject alternative name extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_alternative_name: Option<String>,
    /// Issuer alternative name extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_alternative_name: Option<String>,
    /// Subject directory attributes extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_directory_attributes: Option<String>,
    /// CRL distribution points extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crl_distribution_points: Option<String>,
    /// Inhibit any-policy extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inhibit_any_policy: Option<String>,
    /// Private key usage period (not before).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_usage_period_not_before: Option<Timestamp>,
    /// Private key usage period (not after).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_usage_period_not_after: Option<Timestamp>,
    /// Certificate policies extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_policies: Option<String>,
    /// Policy mappings extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_mappings: Option<String>,
}

/// Windows PE Optional Header Type.
///
/// Contains fields from the PE file optional header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowsPeOptionalHeaderType {
    /// The magic number identifying the image file format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magic_hex: Option<String>,
    /// The major version number of the linker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major_linker_version: Option<u8>,
    /// The minor version number of the linker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor_linker_version: Option<u8>,
    /// The size of the code (text) section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_code: Option<u64>,
    /// The size of the initialized data section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_initialized_data: Option<u64>,
    /// The size of the uninitialized data section (BSS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_uninitialized_data: Option<u64>,
    /// The address of the entry point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_of_entry_point: Option<u64>,
    /// The address that is relative to the image base of the beginning-of-code section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_of_code: Option<u64>,
    /// The address that is relative to the image base of the beginning-of-data section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_of_data: Option<u64>,
    /// The preferred address of the first byte of image when loaded into memory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_base: Option<u64>,
    /// The alignment (in bytes) of sections when they are loaded into memory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_alignment: Option<u64>,
    /// The alignment factor (in bytes) used to align raw data of sections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_alignment: Option<u64>,
    /// The major version number of the required operating system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major_os_version: Option<u16>,
    /// The minor version number of the required operating system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor_os_version: Option<u16>,
    /// The major version number of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major_image_version: Option<u16>,
    /// The minor version number of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor_image_version: Option<u16>,
    /// The major version number of the subsystem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major_subsystem_version: Option<u16>,
    /// The minor version number of the subsystem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor_subsystem_version: Option<u16>,
    /// Reserved, must be zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win32_version_value_hex: Option<String>,
    /// The size (in bytes) of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_image: Option<u64>,
    /// The combined size of MS-DOS stub, PE header, and section headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_headers: Option<u64>,
    /// The image file checksum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_hex: Option<String>,
    /// The subsystem required to run this image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsystem_hex: Option<String>,
    /// DLL characteristics flags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dll_characteristics_hex: Option<String>,
    /// The size of the stack to reserve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_stack_reserve: Option<u64>,
    /// The size of the stack to commit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_stack_commit: Option<u64>,
    /// The size of the local heap space to reserve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_heap_reserve: Option<u64>,
    /// The size of the local heap space to commit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_of_heap_commit: Option<u64>,
    /// Reserved, must be zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader_flags_hex: Option<String>,
    /// The number of data-directory entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_rva_and_sizes: Option<u64>,
    /// Hashes of the optional header.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub hashes: IndexMap<String, String>,
}

impl Default for WindowsPeOptionalHeaderType {
    fn default() -> Self {
        Self {
            magic_hex: None,
            major_linker_version: None,
            minor_linker_version: None,
            size_of_code: None,
            size_of_initialized_data: None,
            size_of_uninitialized_data: None,
            address_of_entry_point: None,
            base_of_code: None,
            base_of_data: None,
            image_base: None,
            section_alignment: None,
            file_alignment: None,
            major_os_version: None,
            minor_os_version: None,
            major_image_version: None,
            minor_image_version: None,
            major_subsystem_version: None,
            minor_subsystem_version: None,
            win32_version_value_hex: None,
            size_of_image: None,
            size_of_headers: None,
            checksum_hex: None,
            subsystem_hex: None,
            dll_characteristics_hex: None,
            size_of_stack_reserve: None,
            size_of_stack_commit: None,
            size_of_heap_reserve: None,
            size_of_heap_commit: None,
            loader_flags_hex: None,
            number_of_rva_and_sizes: None,
            hashes: IndexMap::new(),
        }
    }
}

impl Constrained for WindowsPeOptionalHeaderType {
    /// Validate WindowsPEOptionalHeaderType constraints.
    ///
    /// At least one property must be present.
    fn validate_constraints(&self) -> Result<()> {
        let has_content = self.magic_hex.is_some()
            || self.major_linker_version.is_some()
            || self.minor_linker_version.is_some()
            || self.size_of_code.is_some()
            || self.size_of_initialized_data.is_some()
            || self.size_of_uninitialized_data.is_some()
            || self.address_of_entry_point.is_some()
            || self.base_of_code.is_some()
            || self.base_of_data.is_some()
            || self.image_base.is_some()
            || self.section_alignment.is_some()
            || self.file_alignment.is_some()
            || self.major_os_version.is_some()
            || self.minor_os_version.is_some()
            || self.major_image_version.is_some()
            || self.minor_image_version.is_some()
            || self.major_subsystem_version.is_some()
            || self.minor_subsystem_version.is_some()
            || self.win32_version_value_hex.is_some()
            || self.size_of_image.is_some()
            || self.size_of_headers.is_some()
            || self.checksum_hex.is_some()
            || self.subsystem_hex.is_some()
            || self.dll_characteristics_hex.is_some()
            || self.size_of_stack_reserve.is_some()
            || self.size_of_stack_commit.is_some()
            || self.size_of_heap_reserve.is_some()
            || self.size_of_heap_commit.is_some()
            || self.loader_flags_hex.is_some()
            || self.number_of_rva_and_sizes.is_some()
            || !self.hashes.is_empty();

        if !has_content {
            return Err(Error::AtLeastOneRequired(vec![
                "At least one property must be present in WindowsPEOptionalHeaderType".to_string(),
            ]));
        }
        Ok(())
    }
}

/// Extension definition for custom extensions.
///
/// The ExtensionDefinition object captures the meaning and behavior
/// of a custom extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionDefinition {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    /// Name of the extension (required).
    pub name: String,
    /// Description of the extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Created timestamp (required).
    pub created: Timestamp,
    /// Modified timestamp (required).
    pub modified: Timestamp,
    /// Created by reference (required for Extension Definition).
    pub created_by_ref: Identifier,
    /// Schema for the extension (required).
    pub schema: String,
    /// Version of the extension (required).
    pub version: String,
    /// Extension types (required, at least one).
    pub extension_types: Vec<String>,
    /// External references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_references: Vec<Value>,
    /// Object marking references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,
    /// Granular markings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_pe_binary_ext() {
        let ext = WindowsPeBinaryExt {
            pe_type: "exe".to_string(),
            imphash: Some("abc123".to_string()),
            machine_hex: None,
            number_of_sections: Some(5),
            time_date_stamp: None,
            pointer_to_symbol_table_hex: None,
            number_of_symbols: None,
            size_of_optional_header: None,
            characteristics_hex: None,
            file_header_hashes: IndexMap::new(),
            optional_header: None,
            sections: Vec::new(),
        };

        let json = serde_json::to_string(&ext).unwrap();
        assert!(json.contains("exe"));
    }

    #[test]
    fn test_email_mime_component() {
        let mime = EmailMimeComponent {
            content_type: Some("text/plain".to_string()),
            content_disposition: Some("inline".to_string()),
            body: Some("Hello, World!".to_string()),
            body_raw_ref: None,
        };

        let json = serde_json::to_string(&mime).unwrap();
        assert!(json.contains("text/plain"));
    }

    #[test]
    fn test_x509_v3_extensions() {
        let ext = X509V3ExtensionsType {
            basic_constraints: Some("CA:TRUE".to_string()),
            key_usage: Some("digitalSignature, keyEncipherment".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&ext).unwrap();
        assert!(json.contains("CA:TRUE"));
    }
}
