//! Error type for validated TerraKit core primitives.

use std::fmt;

use crate::region::LodLevel;

/// Errors produced while constructing or mutating core TerraKit primitives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// A grid, extent, or terrain raster was given a zero-sized axis.
    ZeroDimension,

    /// Dimension multiplication exceeded the supported address space.
    DimensionOverflow,

    /// A caller supplied a buffer whose length does not match its declared shape.
    InvalidBufferLength {
        /// Number of values required by the declared extent.
        expected: usize,

        /// Number of values supplied by the caller.
        actual: usize,
    },

    /// A spatial transform contains non-finite, zero-length, or degenerate axes.
    InvalidTransform,

    /// Region spacing on the named axis was non-finite, zero, negative, or made coverage invalid.
    InvalidSpacing {
        /// Axis whose spacing failed validation.
        axis: &'static str,
    },

    /// The selected detail level cannot be resolved by the current region layout.
    UnsupportedLod {
        /// Requested detail tier that cannot be represented.
        lod: LodLevel,
    },

    /// The current default spatial policy could not represent the scale associated
    /// with the selected detail level.
    LodScaleOverflow {
        /// Requested detail tier whose scale overflowed.
        lod: LodLevel,
    },

    /// A region dimension cannot be evenly reduced by the current spatial policy
    /// for the selected detail level.
    LodNotDivisible {
        /// Axis whose cell count was not divisible.
        axis: &'static str,

        /// Configured highest-detail cell count on the axis.
        cell_count: u32,

        /// Current policy scale required by the requested detail tier.
        scale: u64,
    },

    /// A region coordinate resolved to a non-finite world-space origin.
    NonFiniteRegionOrigin,

    /// A 2D coordinate was outside the bounds of the target grid.
    CoordinateOutOfBounds2 {
        /// Requested X coordinate.
        x: usize,

        /// Requested Y coordinate.
        y: usize,

        /// Valid grid width.
        width: usize,

        /// Valid grid height.
        height: usize,
    },

    /// A 3D coordinate was outside the bounds of the target grid.
    CoordinateOutOfBounds3 {
        /// Requested X coordinate.
        x: usize,

        /// Requested Y coordinate.
        y: usize,

        /// Requested Z coordinate.
        z: usize,

        /// Valid grid width.
        width: usize,

        /// Valid grid height.
        height: usize,

        /// Valid grid depth.
        depth: usize,
    },

    /// A height sample was NaN or infinite.
    NonFiniteHeightSample,

    /// A density sample was NaN or infinite.
    NonFiniteDensitySample,

    /// A height axis was NaN, infinite, or zero length.
    InvalidHeightAxis,

    /// A mesh world origin was NaN or infinite.
    NonFiniteMeshOrigin,

    /// A mesh contains more vertices than the canonical mesh index type supports.
    MeshVertexCountOverflow {
        /// Vertex count that could not be addressed by `MeshIndex`.
        vertex_count: usize,
    },

    /// A mesh position, normal, or texture coordinate was NaN or infinite.
    NonFiniteMeshAttribute {
        /// Attribute buffer containing a NaN or infinite value.
        attribute: &'static str,
    },

    /// A mesh index referenced a missing vertex.
    InvalidMeshIndex {
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

    /// A per-vertex attribute did not have one value per vertex.
    InvalidAttributeLength {
        /// Attribute buffer whose length did not match `positions`.
        attribute: &'static str,

        /// Required number of attribute values.
        expected: usize,

        /// Actual number of attribute values.
        actual: usize,
    },
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDimension => {
                write!(formatter, "grid dimensions must be greater than zero")
            }

            Self::DimensionOverflow => {
                write!(formatter, "grid dimensions exceed supported memory limits")
            }

            Self::InvalidBufferLength { expected, actual } => {
                write!(
                    formatter,
                    "invalid buffer length: expected {expected}, received {actual}"
                )
            }

            Self::InvalidTransform => {
                write!(formatter, "spatial transform is invalid")
            }

            Self::InvalidSpacing { axis } => {
                write!(formatter, "region spacing on axis {axis} is invalid")
            }

            Self::UnsupportedLod { lod } => {
                write!(
                    formatter,
                    "detail level {} cannot be resolved by the current region layout",
                    lod.value()
                )
            }

            Self::LodScaleOverflow { lod } => {
                write!(
                    formatter,
                    "current default spatial scale for detail level {} exceeds supported limits",
                    lod.value()
                )
            }

            Self::LodNotDivisible {
                axis,
                cell_count,
                scale,
            } => {
                write!(
                    formatter,
                    "region cell count {cell_count} on axis {axis} is not divisible by current spatial scale {scale}"
                )
            }

            Self::NonFiniteRegionOrigin => {
                write!(
                    formatter,
                    "region coordinate resolved to a non-finite origin"
                )
            }

            Self::CoordinateOutOfBounds2 {
                x,
                y,
                width,
                height,
            } => {
                write!(
                    formatter,
                    "coordinate ({x}, {y}) is outside 2D extent {width}x{height}"
                )
            }

            Self::CoordinateOutOfBounds3 {
                x,
                y,
                z,
                width,
                height,
                depth,
            } => {
                write!(
                    formatter,
                    "coordinate ({x}, {y}, {z}) is outside 3D extent {width}x{height}x{depth}"
                )
            }

            Self::NonFiniteHeightSample => {
                write!(formatter, "height samples must be finite")
            }

            Self::NonFiniteDensitySample => {
                write!(formatter, "density samples must be finite")
            }

            Self::InvalidHeightAxis => {
                write!(formatter, "height axis must be finite and non-zero")
            }

            Self::NonFiniteMeshOrigin => {
                write!(formatter, "mesh origin must be finite")
            }

            Self::MeshVertexCountOverflow { vertex_count } => {
                write!(
                    formatter,
                    "mesh vertex count {vertex_count} exceeds supported mesh index capacity"
                )
            }

            Self::NonFiniteMeshAttribute { attribute } => {
                write!(formatter, "{attribute} values must be finite")
            }

            Self::InvalidMeshIndex {
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

            Self::InvalidAttributeLength {
                attribute,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "{attribute} length must match vertex count: \
                     expected {expected}, received {actual}"
                )
            }
        }
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_useful_context_for_buffer_lengths() {
        let error = CoreError::InvalidBufferLength {
            expected: 4,
            actual: 3,
        };

        assert_eq!(
            error.to_string(),
            "invalid buffer length: expected 4, received 3"
        );
    }

    #[test]
    fn display_includes_useful_context_for_out_of_bounds_coordinates() {
        let error = CoreError::CoordinateOutOfBounds2 {
            x: 4,
            y: 1,
            width: 4,
            height: 2,
        };

        assert_eq!(
            error.to_string(),
            "coordinate (4, 1) is outside 2D extent 4x2"
        );
    }

    #[test]
    fn display_includes_useful_context_for_3d_out_of_bounds_coordinates() {
        let error = CoreError::CoordinateOutOfBounds3 {
            x: 0,
            y: 2,
            z: 1,
            width: 2,
            height: 2,
            depth: 2,
        };

        assert_eq!(
            error.to_string(),
            "coordinate (0, 2, 1) is outside 3D extent 2x2x2"
        );
    }

    #[test]
    fn display_includes_useful_context_for_lod_errors() {
        let error = CoreError::LodNotDivisible {
            axis: "x",
            cell_count: 30,
            scale: 8,
        };

        assert_eq!(
            error.to_string(),
            "region cell count 30 on axis x is not divisible by current spatial scale 8"
        );
    }
}
