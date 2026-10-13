//! Validated terrain mesh resource.
//!
//! Terrain meshes keep a high-precision world origin and compact local vertex
//! buffers so engine adapters can preserve precision in large worlds.

use crate::{
    error::CoreError,
    math::{Vector2F32, Vector3F32, Vector3F64},
};

/// A triangle index into `TerrainMesh::positions`.
pub type MeshIndex = u32;

/// Owned mesh buffers used to construct or decompose a `TerrainMesh`.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainMeshParts {
    /// High-precision world-space origin for the local vertex buffers.
    pub origin: Vector3F64,

    /// Local vertex positions relative to `origin`.
    pub positions: Vec<Vector3F32>,

    /// Triangle indices into `positions`, stored in groups of three.
    pub indices: Vec<MeshIndex>,

    /// Optional per-vertex normals matching `positions` length.
    pub normals: Option<Vec<Vector3F32>>,

    /// Optional per-vertex texture coordinates matching `positions` length.
    pub texcoords: Option<Vec<Vector2F32>>,
}

/// A validated indexed triangle mesh for terrain surfaces.
///
/// The mesh origin is stored as a high-precision world-space `f64` position.
/// Vertex positions are compact `f32` values relative to that origin. Engine
/// adapters should place their object at [`Self::origin`] and upload local
/// vertices unchanged, preserving precision in large worlds.
///
/// Positions, normals, and texture coordinates must be finite. Indices are
/// contiguous `u32` values stored in groups of three and must reference
/// existing vertices. Optional normals are contiguous `Vector3F32` values and
/// optional texture coordinates are contiguous `Vector2F32` values. When
/// accessed through future foreign-function views, mesh buffers are borrowed
/// from the owning generation result.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainMesh {
    origin: Vector3F64,
    positions: Vec<Vector3F32>,
    indices: Vec<MeshIndex>,
    normals: Option<Vec<Vector3F32>>,
    texcoords: Option<Vec<Vector2F32>>,
}

impl TerrainMesh {
    /// Creates a validated terrain mesh.
    ///
    /// `origin` is the world-space mesh origin, `positions` are local `f32`
    /// vertices, `indices` are triangle indices into `positions`, and optional
    /// `normals` and `texcoords` must have one value per vertex.
    pub fn new(
        origin: Vector3F64,
        positions: Vec<Vector3F32>,
        indices: Vec<MeshIndex>,
        normals: Option<Vec<Vector3F32>>,
        texcoords: Option<Vec<Vector2F32>>,
    ) -> Result<Self, CoreError> {
        Self::from_parts(TerrainMeshParts {
            origin,
            positions,
            indices,
            normals,
            texcoords,
        })
    }

    /// Creates a validated terrain mesh with optional per-vertex attributes.
    ///
    /// This is an explicit alias for [`Self::new`] when call sites want to
    /// emphasize `normals` and `texcoords`.
    pub fn with_attributes(
        origin: Vector3F64,
        positions: Vec<Vector3F32>,
        indices: Vec<MeshIndex>,
        normals: Option<Vec<Vector3F32>>,
        texcoords: Option<Vec<Vector2F32>>,
    ) -> Result<Self, CoreError> {
        Self::new(origin, positions, indices, normals, texcoords)
    }

    /// Creates a validated terrain mesh from owned `parts`.
    pub fn from_parts(parts: TerrainMeshParts) -> Result<Self, CoreError> {
        validate_origin(parts.origin)?;
        validate_vertex_count(parts.positions.len())?;
        validate_triangle_indices(&parts.indices)?;
        validate_mesh_indices(&parts.indices, parts.positions.len())?;
        validate_vector3_attribute("positions", &parts.positions)?;

        if let Some(normals) = parts.normals.as_ref() {
            validate_attribute_length("normals", parts.positions.len(), normals.len())?;
            validate_vector3_attribute("normals", normals)?;
        }

        if let Some(texcoords) = parts.texcoords.as_ref() {
            validate_attribute_length("texcoords", parts.positions.len(), texcoords.len())?;
            validate_vector2_attribute("texcoords", texcoords)?;
        }

        Ok(Self {
            origin: parts.origin,
            positions: parts.positions,
            indices: parts.indices,
            normals: parts.normals,
            texcoords: parts.texcoords,
        })
    }

    /// Returns the high-precision world-space mesh origin.
    pub const fn origin(&self) -> Vector3F64 {
        self.origin
    }

    /// Returns contiguous local vertex positions relative to [`Self::origin`].
    pub fn positions(&self) -> &[Vector3F32] {
        &self.positions
    }

    /// Returns contiguous triangle indices into [`Self::positions`].
    ///
    /// Indices are stored as `u32` values in groups of three.
    pub fn indices(&self) -> &[MeshIndex] {
        &self.indices
    }

    /// Returns optional contiguous per-vertex normals.
    pub fn normals(&self) -> Option<&[Vector3F32]> {
        self.normals.as_deref()
    }

    /// Returns optional contiguous per-vertex texture coordinates.
    pub fn texcoords(&self) -> Option<&[Vector2F32]> {
        self.texcoords.as_deref()
    }

    /// Returns the number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the number of triangle indices in the mesh.
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Returns the number of triangles represented by the index buffer.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Returns true when the mesh contains no triangles.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Consumes the mesh and returns its owned buffers.
    pub fn into_parts(self) -> TerrainMeshParts {
        TerrainMeshParts {
            origin: self.origin,
            positions: self.positions,
            indices: self.indices,
            normals: self.normals,
            texcoords: self.texcoords,
        }
    }
}

fn validate_origin(origin: Vector3F64) -> Result<(), CoreError> {
    if !origin.is_finite() {
        return Err(CoreError::NonFiniteMeshOrigin);
    }

    Ok(())
}

fn validate_vertex_count(vertex_count: usize) -> Result<(), CoreError> {
    if vertex_count > u32::MAX as usize {
        return Err(CoreError::MeshVertexCountOverflow { vertex_count });
    }

    Ok(())
}

fn validate_triangle_indices(indices: &[MeshIndex]) -> Result<(), CoreError> {
    if !indices.len().is_multiple_of(3) {
        return Err(CoreError::InvalidTriangleIndexCount {
            index_count: indices.len(),
        });
    }

    Ok(())
}

fn validate_mesh_indices(indices: &[MeshIndex], vertex_count: usize) -> Result<(), CoreError> {
    for &index in indices {
        if usize::try_from(index).map_or(true, |index| index >= vertex_count) {
            return Err(CoreError::InvalidMeshIndex {
                index,
                vertex_count,
            });
        }
    }

    Ok(())
}

fn validate_vector3_attribute(
    attribute: &'static str,
    values: &[Vector3F32],
) -> Result<(), CoreError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(CoreError::NonFiniteMeshAttribute { attribute });
    }

    Ok(())
}

fn validate_vector2_attribute(
    attribute: &'static str,
    values: &[Vector2F32],
) -> Result<(), CoreError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(CoreError::NonFiniteMeshAttribute { attribute });
    }

    Ok(())
}

fn validate_attribute_length(
    attribute: &'static str,
    expected: usize,
    actual: usize,
) -> Result<(), CoreError> {
    if actual != expected {
        return Err(CoreError::InvalidAttributeLength {
            attribute,
            expected,
            actual,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin() -> Vector3F64 {
        Vector3F64::new(1000.0, 20.0, -3000.0)
    }

    fn triangle_positions() -> Vec<Vector3F32> {
        vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(0.0, 1.0, 0.0),
        ]
    }

    #[test]
    fn mesh_accepts_valid_triangle_data() {
        let mesh =
            TerrainMesh::new(origin(), triangle_positions(), vec![0, 1, 2], None, None).unwrap();

        assert_eq!(mesh.origin(), origin());
        assert_eq!(mesh.triangle_count(), 1);
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.index_count(), 3);
        assert_eq!(mesh.indices(), &[0, 1, 2]);
        assert_eq!(mesh.positions(), triangle_positions());
    }

    #[test]
    fn mesh_rejects_non_triangle_index_counts() {
        let error =
            TerrainMesh::new(origin(), triangle_positions(), vec![0, 1], None, None).unwrap_err();

        assert_eq!(
            error,
            CoreError::InvalidTriangleIndexCount { index_count: 2 }
        );
    }

    #[test]
    fn mesh_rejects_indices_outside_positions() {
        let error = TerrainMesh::new(origin(), triangle_positions(), vec![0, 1, 3], None, None)
            .unwrap_err();

        assert_eq!(
            error,
            CoreError::InvalidMeshIndex {
                index: 3,
                vertex_count: 3,
            }
        );
    }

    #[test]
    fn mesh_rejects_attribute_length_mismatches() {
        let error = TerrainMesh::with_attributes(
            origin(),
            triangle_positions(),
            vec![0, 1, 2],
            Some(vec![Vector3F32::ZERO]),
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            CoreError::InvalidAttributeLength {
                attribute: "normals",
                expected: 3,
                actual: 1,
            }
        );
    }

    #[test]
    fn mesh_rejects_non_finite_origin() {
        let error = TerrainMesh::new(
            Vector3F64::new(f64::NAN, 0.0, 0.0),
            triangle_positions(),
            vec![0, 1, 2],
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(error, CoreError::NonFiniteMeshOrigin);
    }

    #[test]
    fn mesh_rejects_non_finite_positions() {
        let mut positions = triangle_positions();
        positions[1] = Vector3F32::new(f32::NAN, 0.0, 0.0);

        let error = TerrainMesh::new(origin(), positions, vec![0, 1, 2], None, None).unwrap_err();

        assert_eq!(
            error,
            CoreError::NonFiniteMeshAttribute {
                attribute: "positions",
            }
        );
    }

    #[test]
    fn mesh_rejects_non_finite_normals() {
        let error = TerrainMesh::with_attributes(
            origin(),
            triangle_positions(),
            vec![0, 1, 2],
            Some(vec![
                Vector3F32::Y,
                Vector3F32::new(0.0, f32::INFINITY, 0.0),
                Vector3F32::Y,
            ]),
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            CoreError::NonFiniteMeshAttribute {
                attribute: "normals",
            }
        );
    }

    #[test]
    fn mesh_rejects_non_finite_texcoords() {
        let error = TerrainMesh::with_attributes(
            origin(),
            triangle_positions(),
            vec![0, 1, 2],
            None,
            Some(vec![
                Vector2F32::ZERO,
                Vector2F32::new(f32::INFINITY, 0.0),
                Vector2F32::new(1.0, 1.0),
            ]),
        )
        .unwrap_err();

        assert_eq!(
            error,
            CoreError::NonFiniteMeshAttribute {
                attribute: "texcoords",
            }
        );
    }

    #[test]
    fn mesh_can_be_decomposed_into_owned_parts() {
        let mesh =
            TerrainMesh::new(origin(), triangle_positions(), vec![0, 1, 2], None, None).unwrap();
        let parts = mesh.into_parts();

        assert_eq!(parts.origin, origin());
        assert_eq!(parts.positions.len(), 3);
        assert_eq!(parts.indices, vec![0, 1, 2]);
        assert_eq!(parts.normals, None);
        assert_eq!(parts.texcoords, None);
    }
}
