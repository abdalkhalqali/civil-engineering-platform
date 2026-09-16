//! Errors produced during project serialisation and deserialisation.

use std::fmt;

/// A failed project operation.
///
/// This is deliberately separate from [`crate::error::ModelError`]: model
/// errors describe broken invariants in the engineering data, while project
/// errors describe problems with the persistence layer (bad headers, wrong
/// format version, serialisation failures).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectError {
    /// The file header is missing or malformed.
    InvalidHeader {
        /// Why the header is invalid.
        reason: String,
    },
    /// The file was written with a format version this kernel does not support.
    UnsupportedFormatVersion {
        /// Version found in the file.
        found: u32,
        /// Maximum version this kernel supports.
        supported: u32,
    },
    /// The file was written with a model schema version this kernel does not
    /// support.
    UnsupportedModelSchema {
        /// Schema version found in the file.
        found: u32,
        /// Maximum schema version this kernel supports.
        supported: u32,
    },
    /// The payload could not be decoded (corrupt data).
    InvalidPayload {
        /// Description of the decoding error.
        reason: String,
    },
    /// The decoded data violates engineering model invariants.
    ModelValidationFailed {
        /// Number of validation issues found.
        issue_count: usize,
    },
    /// Serialisation failed.
    SerializationFailed {
        /// Description of the serialisation error.
        reason: String,
    },
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::InvalidHeader { reason } => {
                write!(f, "invalid project file header: {reason}")
            }
            ProjectError::UnsupportedFormatVersion { found, supported } => {
                write!(
                    f,
                    "unsupported format version {found} (kernel supports up to {supported})"
                )
            }
            ProjectError::UnsupportedModelSchema { found, supported } => {
                write!(
                    f,
                    "unsupported model schema version {found} (kernel supports up to {supported})"
                )
            }
            ProjectError::InvalidPayload { reason } => {
                write!(f, "invalid project payload: {reason}")
            }
            ProjectError::ModelValidationFailed { issue_count } => {
                write!(f, "model validation failed with {issue_count} issue(s)")
            }
            ProjectError::SerializationFailed { reason } => {
                write!(f, "project serialisation failed: {reason}")
            }
        }
    }
}

impl std::error::Error for ProjectError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_are_human_readable() {
        let err = ProjectError::UnsupportedFormatVersion {
            found: 99,
            supported: 1,
        };
        let msg = err.to_string();
        assert!(msg.contains("99"));
        assert!(msg.contains("1"));
    }
}
