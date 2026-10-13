//! Smooth normal generation for indexed triangle meshes.

use terrakit_core::{MeshIndex, Vector3F32};

use crate::AlgorithmError;

/// Generates smooth vertex normals for an indexed triangle mesh.
///
/// Each triangle contributes its unnormalised face normal to all three
/// vertices, so larger triangles naturally have more influence. Degenerate
/// triangles contribute no normal. Vertices that receive no valid contribution
/// are assigned TerraKit's stable positive-Y fallback normal.
pub fn generate_smooth_normals(
    positions: &[Vector3F32],
    indices: &[MeshIndex],
) -> Result<Vec<Vector3F32>, AlgorithmError> {
    validate_positions(positions)?;
    validate_triangle_indices(indices)?;
    validate_vertex_capacity(positions.len())?;
    validate_indices(indices, positions.len())?;

    let mut normals = vec![Vector3F32::ZERO; positions.len()];

    for (triangle_index, triangle) in indices.chunks_exact(3).enumerate() {
        let a_index = checked_index(triangle[0], positions.len())?;
        let b_index = checked_index(triangle[1], positions.len())?;
        let c_index = checked_index(triangle[2], positions.len())?;
        let a = positions[a_index];
        let b = positions[b_index];
        let c = positions[c_index];
        let face_normal = (b - a).cross(c - a);

        if !face_normal.is_finite() {
            return Err(AlgorithmError::NonFiniteMeshNormal {
                index: triangle_index,
            });
        }

        if face_normal.length_squared() == 0.0 {
            continue;
        }

        add_face_normal(&mut normals, a_index, face_normal)?;
        add_face_normal(&mut normals, b_index, face_normal)?;
        add_face_normal(&mut normals, c_index, face_normal)?;
    }

    for (index, normal) in normals.iter_mut().enumerate() {
        *normal = normal.normalized().unwrap_or(Vector3F32::Y);

        if !normal.is_finite() {
            return Err(AlgorithmError::NonFiniteMeshNormal { index });
        }
    }

    Ok(normals)
}

fn validate_positions(positions: &[Vector3F32]) -> Result<(), AlgorithmError> {
    for (index, position) in positions.iter().enumerate() {
        if !position.is_finite() {
            return Err(AlgorithmError::NonFiniteMeshPosition { index });
        }
    }

    Ok(())
}

fn validate_triangle_indices(indices: &[MeshIndex]) -> Result<(), AlgorithmError> {
    if !indices.len().is_multiple_of(3) {
        return Err(AlgorithmError::InvalidTriangleIndexCount {
            index_count: indices.len(),
        });
    }

    Ok(())
}

fn validate_vertex_capacity(vertex_count: usize) -> Result<(), AlgorithmError> {
    if vertex_count > MeshIndex::MAX as usize {
        return Err(AlgorithmError::MeshVertexCountOverflow);
    }

    Ok(())
}

fn validate_indices(indices: &[MeshIndex], vertex_count: usize) -> Result<(), AlgorithmError> {
    for &index in indices {
        checked_index(index, vertex_count)?;
    }

    Ok(())
}

fn checked_index(index: MeshIndex, vertex_count: usize) -> Result<usize, AlgorithmError> {
    let converted = usize::try_from(index).map_err(|_| AlgorithmError::MeshIndexOutOfBounds {
        index,
        vertex_count,
    })?;

    if converted >= vertex_count {
        return Err(AlgorithmError::MeshIndexOutOfBounds {
            index,
            vertex_count,
        });
    }

    Ok(converted)
}

fn add_face_normal(
    normals: &mut [Vector3F32],
    index: usize,
    face_normal: Vector3F32,
) -> Result<(), AlgorithmError> {
    normals[index] += face_normal;

    if !normals[index].is_finite() {
        return Err(AlgorithmError::NonFiniteMeshNormal { index });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close_vector(actual: Vector3F32, expected: Vector3F32) {
        const EPSILON: f32 = 1.0e-6;

        assert!((actual.x - expected.x).abs() <= EPSILON);
        assert!((actual.y - expected.y).abs() <= EPSILON);
        assert!((actual.z - expected.z).abs() <= EPSILON);
    }

    #[test]
    fn flat_xz_plane_produces_positive_y_normals() {
        let positions = vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(0.0, 0.0, 1.0),
            Vector3F32::new(1.0, 0.0, 1.0),
        ];
        let indices = vec![0, 2, 1, 1, 2, 3];
        let normals = generate_smooth_normals(&positions, &indices).unwrap();

        assert_eq!(normals, vec![Vector3F32::Y; 4]);
    }

    #[test]
    fn tilted_triangle_produces_expected_direction() {
        let positions = vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(0.0, 1.0, 0.0),
            Vector3F32::new(1.0, 0.0, 0.0),
        ];
        let normals = generate_smooth_normals(&positions, &[0, 1, 2]).unwrap();

        assert_eq!(normals, vec![Vector3F32::new(0.0, 0.0, -1.0); 3]);
    }

    #[test]
    fn shared_vertices_receive_averaged_normals() {
        let positions = vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(0.0, 0.0, 1.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(0.0, 1.0, 0.0),
        ];
        let normals = generate_smooth_normals(&positions, &[0, 1, 2, 0, 3, 1]).unwrap();
        let averaged = Vector3F32::new(
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        );

        assert_close_vector(normals[0], averaged);
        assert_close_vector(normals[1], averaged);
        assert_eq!(normals[2], Vector3F32::Y);
        assert_eq!(normals[3], Vector3F32::X);
    }

    #[test]
    fn degenerate_triangles_use_positive_y_fallback() {
        let positions = vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(2.0, 0.0, 0.0),
        ];
        let normals = generate_smooth_normals(&positions, &[0, 1, 2]).unwrap();

        assert_eq!(normals, vec![Vector3F32::Y; 3]);
    }

    #[test]
    fn invalid_index_counts_fail() {
        let positions = vec![Vector3F32::ZERO; 3];

        assert_eq!(
            generate_smooth_normals(&positions, &[0, 1]),
            Err(AlgorithmError::InvalidTriangleIndexCount { index_count: 2 })
        );
    }

    #[test]
    fn out_of_range_indices_fail() {
        let positions = vec![Vector3F32::ZERO; 3];

        assert_eq!(
            generate_smooth_normals(&positions, &[0, 1, 3]),
            Err(AlgorithmError::MeshIndexOutOfBounds {
                index: 3,
                vertex_count: 3,
            })
        );
    }

    #[test]
    fn non_finite_positions_fail() {
        let positions = vec![
            Vector3F32::ZERO,
            Vector3F32::new(f32::NAN, 0.0, 0.0),
            Vector3F32::X,
        ];

        assert_eq!(
            generate_smooth_normals(&positions, &[0, 1, 2]),
            Err(AlgorithmError::NonFiniteMeshPosition { index: 1 })
        );
    }

    #[test]
    fn output_length_equals_vertex_count() {
        let positions = vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(0.0, 0.0, 1.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(10.0, 0.0, 0.0),
        ];
        let normals = generate_smooth_normals(&positions, &[0, 1, 2]).unwrap();

        assert_eq!(normals.len(), positions.len());
        assert_eq!(normals[3], Vector3F32::Y);
    }
}
