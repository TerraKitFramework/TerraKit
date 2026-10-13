//! Reusable mesh-generation helpers.
//!
//! Regular-grid helpers assume point samples are stored in X-contiguous row
//! order: `index = y * width + x`.

mod grid_indices;
mod normals;
mod texcoords;

pub use grid_indices::generate_grid_triangle_indices;
pub use normals::generate_smooth_normals;
pub use texcoords::generate_grid_texcoords;
