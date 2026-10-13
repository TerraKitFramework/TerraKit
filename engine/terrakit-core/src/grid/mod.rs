//! Grid dimensions, storage, spatial transforms, and typed fields.
//!
//! This module contains reusable 2D and 3D grid foundations used by canonical
//! terrain resources. Storage is dense and X-contiguous; fields add transform
//! and sampling-domain metadata on top of raw grids.

mod extent;
mod field;
mod storage;
mod transform;

pub use extent::{Extent2, Extent3};
pub use field::{Field2, Field3, SamplingDomain};
pub use storage::{Grid2, Grid3};
pub use transform::{GridTransform2, GridTransform3};
