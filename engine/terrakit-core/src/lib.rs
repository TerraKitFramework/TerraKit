//! Core terrain primitives for TerraKit.
//!
//! This crate owns the small, validated data types shared by generation,
//! conversion, runtime, and tool layers. It intentionally stays free of
//! pipeline behavior and renderer/exporter concerns.
//!
//! TerraKit uses a right-handed coordinate system. X and Z form the standard
//! horizontal plane, and Y is the standard upward axis.
//!
//! Conventions:
//!
//! - World-space positions and transforms use `f64`.
//! - Dense samples and local mesh positions use `f32`.
//! - Grid dimensions use `u32`.
//! - Mesh indices use `u32`.
//! - `Grid2` storage uses `index = y * width + x`.
//! - `Grid3` storage uses `index = (z * height + y) * width + x`.
//!
//! Only explicitly marked primitive value types are intended for direct memory
//! layout correspondence with future foreign-function views. The current core
//! candidates are `#[repr(C)]` vector elements (`Vector2F32`, `Vector2F64`,
//! `Vector3F32`, `Vector3F64`) and transparent integer wrappers
//! (`GenerationSeed`, `SeedDomain`, `LodLevel`, `VoxelId`). Complex resources
//! such as `TerrainMesh`, `HeightField`, `DensityField`, `VoxelVolume`, and
//! `TerrainResource` remain Rust containers and should be exposed through
//! opaque handles or explicit view structures rather than direct struct layout.

#![warn(missing_docs)]

/// Structured error types returned by validated core primitives.
pub mod error;
/// Dense grid storage, spatial fields, extents, and grid transforms.
pub mod grid;
/// Small vector math types used by core terrain data.
pub mod math;
/// Region addressing, layout resolution, and generation descriptors.
pub mod region;
/// Deterministic generation seed and seed-domain types.
pub mod seed;
/// Canonical terrain resource types shared by generation stages.
pub mod terrain;

pub use error::CoreError;
pub use grid::{
    Extent2, Extent3, Field2, Field3, Grid2, Grid3, GridTransform2, GridTransform3, SamplingDomain,
};
pub use math::{Vector2F32, Vector2F64, Vector3F32, Vector3F64};
pub use region::{
    GenerationRegion, LodLevel, RegionCoord2, RegionCoord3, RegionDescriptor2, RegionDescriptor3,
    RegionLayout2, RegionLayout3, RegionRequest2, RegionRequest3,
};
pub use seed::{GenerationSeed, SeedDomain};
pub use terrain::{
    DensityField, DensitySample, HeightField, HeightSample, MeshIndex, ScatterPoint, ScatterPoints,
    TerrainMesh, TerrainMeshParts, TerrainResource, VoxelId, VoxelVolume,
};

/// Convenient import surface for common TerraKit core types.
pub mod prelude {
    pub use crate::{
        CoreError, DensityField, DensitySample, Extent2, Extent3, Field2, Field3, GenerationRegion,
        GenerationSeed, Grid2, Grid3, GridTransform2, GridTransform3, HeightField, HeightSample,
        LodLevel, MeshIndex, RegionCoord2, RegionCoord3, RegionDescriptor2, RegionDescriptor3,
        RegionLayout2, RegionLayout3, RegionRequest2, RegionRequest3, SamplingDomain, SeedDomain,
        TerrainMesh, TerrainMeshParts, TerrainResource, Vector2F32, Vector2F64, Vector3F32,
        Vector3F64, VoxelId, VoxelVolume,
    };
}
