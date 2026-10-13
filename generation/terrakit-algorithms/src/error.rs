//! Algorithm validation and sampling errors.

use std::fmt;

/// Errors produced by validated TerraKit algorithm configuration or sampling.
///
/// These errors describe numeric and algorithm-level validation failures only.
/// Pipeline resources, plugins, engine conversions, and runtime scheduling are
/// outside this crate's responsibility.
#[derive(Debug, Clone, PartialEq)]
pub enum AlgorithmError {
    /// An algorithm-space coordinate was NaN or infinite, or became non-finite
    /// while applying validated sampling settings.
    NonFiniteCoordinate,

    /// A named numeric setting was NaN or infinite.
    NonFiniteSetting {
        /// Name of the invalid setting field.
        field: &'static str,
    },

    /// Frequency must be finite, greater than zero, and remain finite across
    /// all configured octaves.
    InvalidFrequency,

    /// Lacunarity must be finite, greater than zero, and keep octave frequency
    /// finite.
    InvalidLacunarity,

    /// Persistence must be finite and non-negative.
    InvalidPersistence,

    /// Amplitude must be finite and keep octave amplitudes and output scale
    /// finite. Zero amplitude is valid; negative amplitude is permitted.
    InvalidAmplitude,

    /// Octave count must be greater than zero and no larger than the supported
    /// safety limit.
    InvalidOctaveCount,

    /// A source noise implementation returned NaN or infinity during validated
    /// fractal sampling.
    NonFiniteSample,

    /// A regular mesh grid needs at least two point samples on each axis.
    InvalidGridExtent {
        /// Width supplied for the regular grid.
        width: u32,

        /// Height supplied for the regular grid.
        height: u32,
    },

    /// A generated mesh would contain more vertices than its index type can address.
    MeshVertexCountOverflow,

    /// A generated mesh would contain more indices than supported memory limits allow.
    MeshIndexCountOverflow,

    /// Mesh triangle indices must reference existing vertices.
    MeshIndexOutOfBounds {
        /// Index value that did not reference an existing vertex.
        index: u32,

        /// Number of vertices available in the mesh.
        vertex_count: usize,
    },

    /// Triangle meshes must store indices in groups of three.
    InvalidTriangleIndexCount {
        /// Number of indices supplied for the triangle list.
        index_count: usize,
    },

    /// A mesh position used by an algorithm was NaN or infinite.
    NonFiniteMeshPosition {
        /// Position index containing a non-finite component.
        index: usize,
    },

    /// A generated mesh normal was NaN or infinite.
    NonFiniteMeshNormal {
        /// Normal index that became non-finite.
        index: usize,
    },
}

impl fmt::Display for AlgorithmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteCoordinate => {
                write!(formatter, "algorithm coordinates must be finite")
            }
            Self::NonFiniteSetting { field } => {
                write!(formatter, "{field} must be finite")
            }
            Self::InvalidFrequency => {
                write!(
                    formatter,
                    "frequency must be greater than zero and remain finite"
                )
            }
            Self::InvalidLacunarity => {
                write!(
                    formatter,
                    "lacunarity must be greater than zero and keep frequency finite"
                )
            }
            Self::InvalidPersistence => {
                write!(formatter, "persistence must be finite and non-negative")
            }
            Self::InvalidAmplitude => {
                write!(
                    formatter,
                    "amplitude must keep octave amplitudes and output scale finite"
                )
            }
            Self::InvalidOctaveCount => {
                write!(formatter, "octaves must be between 1 and 32")
            }
            Self::NonFiniteSample => {
                write!(formatter, "source noise samples must be finite")
            }
            Self::InvalidGridExtent { width, height } => {
                write!(
                    formatter,
                    "regular mesh grids require at least 2x2 samples, received {width}x{height}"
                )
            }
            Self::MeshVertexCountOverflow => {
                write!(
                    formatter,
                    "mesh vertex count exceeds supported mesh index capacity"
                )
            }
            Self::MeshIndexCountOverflow => {
                write!(
                    formatter,
                    "mesh index count exceeds supported memory or address limits"
                )
            }
            Self::MeshIndexOutOfBounds {
                index,
                vertex_count,
            } => {
                write!(
                    formatter,
                    "mesh index {index} exceeds vertex count {vertex_count}"
                )
            }
            Self::InvalidTriangleIndexCount { index_count } => {
                write!(
                    formatter,
                    "triangle mesh index count must be divisible by 3, received {index_count}"
                )
            }
            Self::NonFiniteMeshPosition { index } => {
                write!(formatter, "mesh position {index} must be finite")
            }
            Self::NonFiniteMeshNormal { index } => {
                write!(formatter, "mesh normal {index} must be finite")
            }
        }
    }
}

impl std::error::Error for AlgorithmError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_non_finite_settings() {
        let error = AlgorithmError::NonFiniteSetting { field: "frequency" };

        assert_eq!(error.to_string(), "frequency must be finite");
    }
}
