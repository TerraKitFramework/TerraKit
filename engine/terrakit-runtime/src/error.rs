//! Runtime-level error type and conversions.

use std::{error::Error, fmt};

use terrakit_core::CoreError;
use terrakit_pipeline::PipelineError;

use crate::GenerationRegionKind;

/// Structured errors returned by TerraKit's high-level runtime boundary.
#[derive(Debug)]
pub enum RuntimeError {
    /// The runtime was configured for one region dimensionality, but the request used another.
    LayoutMismatch {
        /// Dimensionality configured on the runtime layout.
        configured: GenerationRegionKind,

        /// Dimensionality requested for this generation call.
        requested: GenerationRegionKind,
    },

    /// The configured core layout could not resolve the lightweight request.
    RegionResolution {
        /// Core error produced by the selected region layout.
        source: CoreError,
    },

    /// The configured terrain pipeline failed while executing stages.
    PipelineExecution {
        /// Pipeline error produced by serial stage execution.
        source: PipelineError,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMismatch {
                configured,
                requested,
            } => write!(
                formatter,
                "configured {} runtime cannot execute a {} generation request",
                region_kind_label(*configured),
                region_kind_label(*requested)
            ),

            Self::RegionResolution { source } => {
                write!(formatter, "failed to resolve generation region: {source}")
            }

            Self::PipelineExecution { source } => {
                write!(formatter, "terrain pipeline execution failed: {source}")
            }
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LayoutMismatch { .. } => None,
            Self::RegionResolution { source } => Some(source),
            Self::PipelineExecution { source } => Some(source),
        }
    }
}

impl From<PipelineError> for RuntimeError {
    fn from(source: PipelineError) -> Self {
        Self::PipelineExecution { source }
    }
}

fn region_kind_label(kind: GenerationRegionKind) -> &'static str {
    match kind {
        GenerationRegionKind::Region2 => "2D",
        GenerationRegionKind::Region3 => "3D",
    }
}
