//! Error types used by stage execution and pipeline validation.

use std::{error::Error, fmt};

use terrakit_core::CoreError;

use crate::{ResourceKey, ResourceKind, StageId};

/// Error reported by a terrain stage while transforming resources.
#[derive(Debug)]
pub struct StageError {
    message: String,
}

impl StageError {
    /// Creates a stage error from a human-readable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the stage error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for StageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for StageError {}

impl From<CoreError> for StageError {
    fn from(error: CoreError) -> Self {
        Self::new(error.to_string())
    }
}

/// Structured errors produced while validating or executing a terrain pipeline.
#[derive(Debug)]
pub enum PipelineError {
    /// A required resource was not present before a stage executed.
    MissingResource {
        /// Stage that was about to execute.
        stage_id: StageId,

        /// Human-readable stage name.
        stage_name: String,

        /// Missing resource key.
        key: ResourceKey,
    },

    /// A resource existed, but it was not the kind required by a stage.
    WrongResourceKind {
        /// Stage that was about to execute.
        stage_id: StageId,

        /// Human-readable stage name.
        stage_name: String,

        /// Resource key whose kind was checked.
        key: ResourceKey,

        /// Resource kind declared by the stage requirement.
        expected: ResourceKind,

        /// Actual resource kind stored at `key`.
        actual: ResourceKind,
    },

    /// A stage declared a create-only product whose key already existed.
    ResourceAlreadyExists {
        /// Stage that was about to execute.
        stage_id: StageId,

        /// Human-readable stage name.
        stage_name: String,

        /// Output key that was already occupied.
        key: ResourceKey,
    },

    /// A stage completed without producing a declared output resource.
    MissingOutput {
        /// Stage that just executed.
        stage_id: StageId,

        /// Human-readable stage name.
        stage_name: String,

        /// Declared output key that was not produced.
        key: ResourceKey,
    },

    /// A stage produced an output resource with the wrong kind.
    WrongOutputKind {
        /// Stage that just executed.
        stage_id: StageId,

        /// Human-readable stage name.
        stage_name: String,

        /// Output key whose kind was checked.
        key: ResourceKey,

        /// Resource kind declared by the stage product.
        expected: ResourceKind,

        /// Actual resource kind stored at `key`.
        actual: ResourceKind,
    },

    /// A stage returned its own execution error.
    StageFailed {
        /// Stage that failed.
        stage_id: StageId,

        /// Human-readable stage name.
        stage_name: String,

        /// Error returned by the stage.
        source: StageError,
    },

    /// A non-stage caller supplied invalid pipeline setup data.
    Configuration {
        /// Human-readable configuration failure.
        message: String,
    },
}

impl PipelineError {
    /// Creates a configuration error for callers that fail before a stage exists.
    pub fn new(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingResource {
                stage_id,
                stage_name,
                key,
            } => write!(
                formatter,
                "stage '{stage_name}' ({stage_id:?}) is missing required resource {key:?}"
            ),

            Self::WrongResourceKind {
                stage_id,
                stage_name,
                key,
                expected,
                actual,
            } => write!(
                formatter,
                "stage '{stage_name}' ({stage_id:?}) expected resource {key:?} to be \
                 {expected:?}, found {actual:?}"
            ),

            Self::ResourceAlreadyExists {
                stage_id,
                stage_name,
                key,
            } => write!(
                formatter,
                "stage '{stage_name}' ({stage_id:?}) cannot create resource {key:?} \
                 because it already exists"
            ),

            Self::MissingOutput {
                stage_id,
                stage_name,
                key,
            } => write!(
                formatter,
                "stage '{stage_name}' ({stage_id:?}) did not produce declared resource {key:?}"
            ),

            Self::WrongOutputKind {
                stage_id,
                stage_name,
                key,
                expected,
                actual,
            } => write!(
                formatter,
                "stage '{stage_name}' ({stage_id:?}) declared output {key:?} as \
                 {expected:?}, but produced {actual:?}"
            ),

            Self::StageFailed {
                stage_id,
                stage_name,
                source,
            } => write!(
                formatter,
                "stage '{stage_name}' ({stage_id:?}) failed: {source}"
            ),

            Self::Configuration { message } => {
                write!(formatter, "pipeline configuration error: {message}")
            }
        }
    }
}

impl Error for PipelineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StageFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}
