//! Triangle-index generation for regular point-sampled grids.

use terrakit_core::MeshIndex;

use crate::AlgorithmError;

/// Generates two triangles for every cell in a regular point-sampled grid.
///
/// Vertices are addressed in X-contiguous row order: `index = y * width + x`.
/// Each cell emits triangles in this order:
///
/// - `top_left, bottom_left, top_right`
/// - `top_right, bottom_left, bottom_right`
///
/// For TerraKit's right-handed XZ heightfield convention with positive Y up,
/// this winding produces upward-facing geometry on a flat plane.
pub fn generate_grid_triangle_indices(
    width: u32,
    height: u32,
) -> Result<Vec<MeshIndex>, AlgorithmError> {
    validate_grid_extent(width, height)?;
    let _vertex_count = checked_mesh_vertex_count(width, height)?;
    let index_count = checked_grid_index_count(width, height)?;

    let mut indices = Vec::new();
    indices
        .try_reserve_exact(index_count)
        .map_err(|_| AlgorithmError::MeshIndexCountOverflow)?;

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let top_left = mesh_index_at(width, x, y)?;
            let top_right = mesh_index_at(width, x + 1, y)?;
            let bottom_left = mesh_index_at(width, x, y + 1)?;
            let bottom_right = mesh_index_at(width, x + 1, y + 1)?;

            indices.extend_from_slice(&[
                top_left,
                bottom_left,
                top_right,
                top_right,
                bottom_left,
                bottom_right,
            ]);
        }
    }

    Ok(indices)
}

pub(super) fn validate_grid_extent(width: u32, height: u32) -> Result<(), AlgorithmError> {
    if width < 2 || height < 2 {
        return Err(AlgorithmError::InvalidGridExtent { width, height });
    }

    Ok(())
}

pub(super) fn checked_mesh_vertex_count(width: u32, height: u32) -> Result<usize, AlgorithmError> {
    let vertex_count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(AlgorithmError::MeshVertexCountOverflow)?;

    if vertex_count > u64::from(MeshIndex::MAX) {
        return Err(AlgorithmError::MeshVertexCountOverflow);
    }

    usize::try_from(vertex_count).map_err(|_| AlgorithmError::MeshVertexCountOverflow)
}

fn checked_grid_index_count(width: u32, height: u32) -> Result<usize, AlgorithmError> {
    let cell_count = u64::from(width - 1)
        .checked_mul(u64::from(height - 1))
        .ok_or(AlgorithmError::MeshIndexCountOverflow)?;
    let index_count = cell_count
        .checked_mul(6)
        .ok_or(AlgorithmError::MeshIndexCountOverflow)?;

    usize::try_from(index_count).map_err(|_| AlgorithmError::MeshIndexCountOverflow)
}

fn mesh_index_at(width: u32, x: u32, y: u32) -> Result<MeshIndex, AlgorithmError> {
    let index = u64::from(y)
        .checked_mul(u64::from(width))
        .and_then(|value| value.checked_add(u64::from(x)))
        .ok_or(AlgorithmError::MeshVertexCountOverflow)?;

    MeshIndex::try_from(index).map_err(|_| AlgorithmError::MeshVertexCountOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrakit_core::Vector3F32;

    #[test]
    fn two_by_two_grid_produces_two_triangles() {
        let indices = generate_grid_triangle_indices(2, 2).unwrap();

        assert_eq!(indices, vec![0, 2, 1, 1, 2, 3]);
    }

    #[test]
    fn three_by_three_grid_produces_eight_triangles() {
        let indices = generate_grid_triangle_indices(3, 3).unwrap();

        assert_eq!(indices.len(), 24);
        assert!(indices.iter().all(|index| *index < 9));
    }

    #[test]
    fn triangle_winding_points_up_on_flat_xz_grid() {
        let positions = [
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(0.0, 0.0, 1.0),
            Vector3F32::new(1.0, 0.0, 1.0),
        ];
        let indices = generate_grid_triangle_indices(2, 2).unwrap();

        for triangle in indices.chunks_exact(3) {
            let a = positions[triangle[0] as usize];
            let b = positions[triangle[1] as usize];
            let c = positions[triangle[2] as usize];
            let normal = (b - a).cross(c - a).normalized().unwrap();

            assert_eq!(normal, Vector3F32::Y);
        }
    }

    #[test]
    fn width_below_two_fails() {
        assert_eq!(
            generate_grid_triangle_indices(1, 2),
            Err(AlgorithmError::InvalidGridExtent {
                width: 1,
                height: 2,
            })
        );
    }

    #[test]
    fn height_below_two_fails() {
        assert_eq!(
            generate_grid_triangle_indices(2, 1),
            Err(AlgorithmError::InvalidGridExtent {
                width: 2,
                height: 1,
            })
        );
    }

    #[test]
    fn vertex_count_overflow_fails_before_allocation() {
        assert_eq!(
            generate_grid_triangle_indices(u32::MAX, u32::MAX),
            Err(AlgorithmError::MeshVertexCountOverflow)
        );
    }
}
