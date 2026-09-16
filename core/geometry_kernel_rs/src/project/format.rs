//! Serialisation format for project files.
//!
//! The trait [`ProjectSerializer`] abstracts the on-disk format so that the
//! engineering model never depends on how bytes are arranged. A concrete JSON
//! implementation ([`ProjectFormatV1`]) is provided; replacing it with SQLite,
//! FlatBuffers, or a binary format later requires only implementing the trait
//! once.
//!
//! # Deterministic representation
//!
//! Both `serialize` and `deserialize` are designed to produce **deterministic
//! output**: the same logical model always yields the same bytes. This is
//! possible because all model collections use [`BTreeMap`](std::collections::BTreeMap)
//! with `Uuid` keys, and `serde_json` serialises them in sorted key order.

use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use super::{Project, ProjectError, PROJECT_FORMAT_VERSION};
use crate::model::EngineeringModel;

/// Trait for project file serialisation.
///
/// Implementors convert a [`Project`] to and from a byte stream. The trait is
/// intentionally simple — a single reader and a single writer — so it works
/// with both in-memory buffers and files.
pub trait ProjectSerializer {
    /// Serialises a project into bytes.
    fn serialize(project: &Project) -> Result<Vec<u8>, ProjectError>;

    /// Deserialises a project from bytes.
    fn deserialize(data: &[u8]) -> Result<Project, ProjectError>;
}

/// The on-disk header of a `.civilx` file.
///
/// The header is always the first thing in the file. It lets a loader determine
/// whether it can handle the file before reading the full payload.
///
/// ```text
/// ┌──────────────────────────────────────────────────┐
/// │  Header                                          │
/// │  ├── format_version: u32                         │
/// │  ├── model_schema_version: u32                   │
/// │  ├── project_id: Uuid                            │
/// │  └── (reserved: u32 = 0)                         │
/// ├──────────────────────────────────────────────────┤
/// │  Payload (format-specific)                       │
/// │  ├── metadata (JSON in v1)                       │
/// │  └── engineering model (JSON in v1)              │
/// └──────────────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectFileHeader {
    /// Version of the `.civilx` file format.
    pub format_version: u32,
    /// Version of the engineering model schema the data carries.
    pub model_schema_version: u32,
    /// Stable identity of the project.
    pub project_id: uuid::Uuid,
    /// Reserved for future use.
    #[serde(default)]
    pub reserved: u32,
}

/// JSON-based serialiser for `.civilx` files.
///
/// The payload layout is:
///
/// ```text
/// header_json  \n  metadata_json  \n  model_json
/// ```
///
/// Each segment is a self-contained JSON value separated by a newline. This
/// makes the file easy to inspect with `jq` and simple to parse without a
/// streaming parser.
pub struct ProjectFormatV1;

impl ProjectFormatV1 {
    /// File extension for this format.
    pub const EXTENSION: &'static str = "civilx";

    /// Maximum size of a project file (100 MiB).
    pub const MAX_SIZE: usize = 100 * 1024 * 1024;
}

impl ProjectSerializer for ProjectFormatV1 {
    fn serialize(project: &Project) -> Result<Vec<u8>, ProjectError> {
        let header = ProjectFileHeader {
            format_version: PROJECT_FORMAT_VERSION,
            model_schema_version: project.model.schema_version,
            project_id: project.project_id(),
            reserved: 0,
        };

        let header_json =
            serde_json::to_string(&header).map_err(|e| ProjectError::SerializationFailed {
                reason: format!("failed to serialise header: {e}"),
            })?;

        let metadata_json = serde_json::to_string(&project.metadata).map_err(|e| {
            ProjectError::SerializationFailed {
                reason: format!("failed to serialise metadata: {e}"),
            }
        })?;

        // Use compact JSON for the model segment to avoid newline-in-model
        // breaking the three-segment newline-delimited format.
        let model_json = serde_json::to_string(&project.model).map_err(|e| {
            ProjectError::SerializationFailed {
                reason: format!("failed to serialise model: {e}"),
            }
        })?;

        let mut buffer =
            Vec::with_capacity(header_json.len() + metadata_json.len() + model_json.len() + 3);
        buffer.extend_from_slice(header_json.as_bytes());
        buffer.push(b'\n');
        buffer.extend_from_slice(metadata_json.as_bytes());
        buffer.push(b'\n');
        buffer.extend_from_slice(model_json.as_bytes());
        buffer.push(b'\n');

        Ok(buffer)
    }

    fn deserialize(data: &[u8]) -> Result<Project, ProjectError> {
        if data.len() > Self::MAX_SIZE {
            return Err(ProjectError::InvalidPayload {
                reason: format!(
                    "file is {} bytes, exceeding the maximum of {} bytes",
                    data.len(),
                    Self::MAX_SIZE
                ),
            });
        }

        // Parse the three newline-separated segments.
        let text = std::str::from_utf8(data).map_err(|e| ProjectError::InvalidPayload {
            reason: format!("file is not valid UTF-8: {e}"),
        })?;

        let mut segments = text.split('\n').filter(|s| !s.is_empty());

        // --- Header ---
        let header_str = segments.next().ok_or_else(|| ProjectError::InvalidHeader {
            reason: "file is empty".to_string(),
        })?;

        let header: ProjectFileHeader =
            serde_json::from_str(header_str).map_err(|e| ProjectError::InvalidHeader {
                reason: format!("cannot parse header: {e}"),
            })?;

        if header.format_version > PROJECT_FORMAT_VERSION {
            return Err(ProjectError::UnsupportedFormatVersion {
                found: header.format_version,
                supported: PROJECT_FORMAT_VERSION,
            });
        }

        if header.model_schema_version > crate::model::MODEL_SCHEMA_VERSION {
            return Err(ProjectError::UnsupportedModelSchema {
                found: header.model_schema_version,
                supported: crate::model::MODEL_SCHEMA_VERSION,
            });
        }

        // --- Metadata ---
        let metadata_str = segments
            .next()
            .ok_or_else(|| ProjectError::InvalidPayload {
                reason: "metadata segment is missing".to_string(),
            })?;

        let metadata: super::ProjectMetadata =
            serde_json::from_str(metadata_str).map_err(|e| ProjectError::InvalidPayload {
                reason: format!("cannot parse metadata: {e}"),
            })?;

        // --- Model ---
        let model_str = segments
            .next()
            .ok_or_else(|| ProjectError::InvalidPayload {
                reason: "model segment is missing".to_string(),
            })?;

        let model: EngineeringModel =
            EngineeringModel::from_json(model_str).map_err(|e| ProjectError::InvalidPayload {
                reason: format!("cannot parse engineering model: {e}"),
            })?;

        // Verify the header's project_id matches the model.
        if header.project_id != model.project_id {
            return Err(ProjectError::InvalidHeader {
                reason: format!(
                    "header project_id ({}) does not match model project_id ({})",
                    header.project_id, model.project_id
                ),
            });
        }

        // Verify the schema version matches.
        if header.model_schema_version != model.schema_version {
            return Err(ProjectError::InvalidHeader {
                reason: format!(
                    "header model_schema_version ({}) does not match model schema_version ({})",
                    header.model_schema_version, model.schema_version
                ),
            });
        }

        Ok(Project { metadata, model })
    }
}

/// Write a project to a `Write` sink.
#[allow(dead_code)]
pub fn save_project_to<W: Write>(writer: &mut W, project: &Project) -> Result<(), ProjectError> {
    let bytes = ProjectFormatV1::serialize(project)?;
    writer
        .write_all(&bytes)
        .map_err(|e| ProjectError::SerializationFailed {
            reason: format!("I/O error during write: {e}"),
        })?;
    Ok(())
}

/// Read a project from a `Read` source.
#[allow(dead_code)]
pub fn load_project_from<R: Read>(reader: &mut R) -> Result<Project, ProjectError> {
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .map_err(|e| ProjectError::InvalidPayload {
            reason: format!("I/O error during read: {e}"),
        })?;
    ProjectFormatV1::deserialize(&buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MODEL_SCHEMA_VERSION;

    #[test]
    fn header_round_trip() {
        let header = ProjectFileHeader {
            format_version: PROJECT_FORMAT_VERSION,
            model_schema_version: MODEL_SCHEMA_VERSION,
            project_id: uuid::Uuid::new_v4(),
            reserved: 0,
        };
        let json = serde_json::to_string(&header).unwrap();
        let restored: ProjectFileHeader = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, header);
    }

    #[test]
    fn empty_project_serialises_and_deserialises() {
        let project = Project::new("Empty Test");
        let bytes = ProjectFormatV1::serialize(&project).unwrap();
        let restored = ProjectFormatV1::deserialize(&bytes).unwrap();
        assert_eq!(restored.project_id(), project.project_id());
        assert_eq!(restored.metadata.name, "Empty Test");
        assert_eq!(restored.model.revision, 0);
    }

    #[test]
    fn format_version_mismatch_is_rejected() {
        let project = Project::new("Version Test");
        let bytes = ProjectFormatV1::serialize(&project).unwrap();

        // Tamper with the format version in the header JSON.
        let header_end = bytes.iter().position(|&b| b == b'\n').unwrap();
        let mut header_bytes = bytes[..header_end].to_vec();
        let header_str = String::from_utf8(header_bytes.clone()).unwrap();
        let tampered = header_str.replace(
            &format!("\"format_version\":{PROJECT_FORMAT_VERSION}"),
            "\"format_version\":999",
        );
        header_bytes = tampered.into_bytes();
        header_bytes.extend_from_slice(&bytes[header_end..]);
        let bytes = header_bytes;

        let result = ProjectFormatV1::deserialize(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectError::UnsupportedFormatVersion { found, .. } => {
                assert_eq!(found, 999);
            }
            other => panic!("expected UnsupportedFormatVersion, got {other:?}"),
        }
    }

    #[test]
    fn empty_file_returns_invalid_header() {
        let result = ProjectFormatV1::deserialize(b"");
        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectError::InvalidHeader { .. } => {}
            other => panic!("expected InvalidHeader, got {other:?}"),
        }
    }

    #[test]
    fn non_utf8_returns_invalid_payload() {
        let result = ProjectFormatV1::deserialize(&[0xFF, 0xFE, 0x00]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectError::InvalidPayload { .. } => {}
            other => panic!("expected InvalidPayload, got {other:?}"),
        }
    }

    #[test]
    fn project_id_mismatch_is_rejected() {
        let project = Project::new("ID Mismatch");
        let bytes = ProjectFormatV1::serialize(&project).unwrap();

        // Replace the header's project_id with a different UUID.
        let header_end = bytes.iter().position(|&b| b == b'\n').unwrap();
        let header_str = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
        let fake_id = uuid::Uuid::new_v4();
        let tampered = header_str.replace(&project.project_id().to_string(), &fake_id.to_string());
        let mut new_bytes = tampered.into_bytes();
        new_bytes.extend_from_slice(&bytes[header_end..]);

        let result = ProjectFormatV1::deserialize(&new_bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectError::InvalidHeader { reason } => {
                assert!(reason.contains("project_id"));
            }
            other => panic!("expected InvalidHeader, got {other:?}"),
        }
    }

    #[test]
    fn io_round_trip() {
        let project = Project::new("IO Test");
        let mut buffer = Vec::new();
        save_project_to(&mut buffer, &project).unwrap();
        let restored = load_project_from(&mut buffer.as_slice()).unwrap();
        assert_eq!(restored, project);
    }
}
