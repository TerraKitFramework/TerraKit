//! Texture-coordinate generation for regular point-sampled grids.

use terrakit_core::Vector2F32;

use crate::AlgorithmError;

use super::grid_indices::{checked_mesh_vertex_count, validate_grid_extent};

/// Generates region-local `0..=1` texture coordinates for a regular grid.
///
/// Vertex order is X-contiguous row order: `index = y * width + x`. The U axis
/// spans grid X and the V axis spans grid Y.
pub fn generate_grid_texcoords(width: u32, height: u32) -> Result<Vec<Vector2F32>, AlgorithmError> {
    validate_grid_extent(width, height)?;
    let vertex_count = checked_mesh_vertex_count(width, height)?;

    let mut texcoords = Vec::new();
    texcoords
        .try_reserve_exact(vertex_count)
        .map_err(|_| AlgorithmError::MeshVertexCountOverflow)?;

    let max_x = (width - 1) as f32;
    let max_y = (height - 1) as f32;

    for y in 0..height {
        for x in 0..width {
            texcoords.push(Vector2F32::new(x as f32 / max_x, y as f32 / max_y));
        }
    }

    Ok(texcoords)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_by_two_grid_spans_all_uv_corners() {
        let texcoords = generate_grid_texcoords(2, 2).unwrap();

        assert_eq!(
            texcoords,
            vec![
                Vector2F32::new(0.0, 0.0),
                Vector2F32::new(1.0, 0.0),
                Vector2F32::new(0.0, 1.0),
                Vector2F32::new(1.0, 1.0),
            ]
        );
    }

    #[test]
    fn values_remain_inside_unit_range() {
        let texcoords = generate_grid_texcoords(5, 3).unwrap();

        assert!(texcoords.iter().all(|texcoord| {
            (0.0..=1.0).contains(&texcoord.x) && (0.0..=1.0).contains(&texcoord.y)
        }));
    }

    #[test]
    fn output_length_equals_vertex_count() {
        let texcoords = generate_grid_texcoords(3, 4).unwrap();

        assert_eq!(texcoords.len(), 12);
    }

    #[test]
    fn invalid_dimensions_fail() {
        assert_eq!(
            generate_grid_texcoords(1, 4),
            Err(AlgorithmError::InvalidGridExtent {
                width: 1,
                height: 4,
            })
        );
    }
}
