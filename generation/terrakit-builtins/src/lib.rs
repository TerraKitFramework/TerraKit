//! Official first-party terrain pipeline stages for TerraKit.
//!
//! Built-in stages are ordinary safe Rust implementations of
//! [`terrakit_pipeline::TerrainStage`]. They orchestrate canonical
//! [`terrakit_core`] resources and delegate reusable procedural calculations to
//! [`terrakit_algorithms`].
//!
//! # Heightfield mesh example
//!
//! ```
//! # use terrakit_algorithms::noise::{FractalSettings, NoiseAlgorithm};
//! # use terrakit_builtins::{FlatHeightStage, HeightFieldMeshStage, HeightNoiseMode, NoiseHeightStage};
//! # use terrakit_core::{SeedDomain, Vector3F64};
//! # use terrakit_pipeline::{ResourceKey, StageId, TerrainPipeline};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! const BASE_HEIGHT: ResourceKey = ResourceKey(100);
//! const NOISY_HEIGHT: ResourceKey = ResourceKey(101);
//!
//! let mut pipeline = TerrainPipeline::new();
//!
//! pipeline.add_stage(FlatHeightStage::new(
//!     StageId(1),
//!     BASE_HEIGHT,
//!     0.0,
//!     Vector3F64::Y,
//! )?);
//! pipeline.add_stage(NoiseHeightStage::new(
//!     StageId(2),
//!     BASE_HEIGHT,
//!     NOISY_HEIGHT,
//!     NoiseAlgorithm::Value,
//!     FractalSettings::default(),
//!     SeedDomain::new(0),
//!     HeightNoiseMode::Add,
//! )?);
//! pipeline.add_stage(HeightFieldMeshStage::new(
//!     StageId(3),
//!     NOISY_HEIGHT,
//!     ResourceKey::MESH,
//! ));
//! # Ok(())
//! # }
//! ```
//!
//! When assembling through discoverable definitions, pass the definition's
//! schema version with the stable stage type ID:
//!
//! ```
//! # use terrakit_builtins::FlatHeightDefinition;
//! # use terrakit_pipeline::{StageConstruction, StageId, StageTypeId};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let construction = StageConstruction::new(
//!     StageId(1),
//!     StageTypeId::try_new(FlatHeightDefinition::TYPE_ID)?,
//!     FlatHeightDefinition::SCHEMA_VERSION,
//! );
//!
//! assert_eq!(construction.schema_version(), FlatHeightDefinition::SCHEMA_VERSION);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

/// Discoverable schemas and factories for first-party built-in stages.
pub mod definition;
/// Height-field stage implementations.
pub mod height;
/// Mesh stage implementations.
pub mod mesh;
/// Object scattering stage implementations.
pub mod scattering;

pub use definition::{
    FlatHeightDefinition, HeightFieldMeshDefinition, NoiseHeightDefinition, builtin_stage_registry,
    register_builtin_stage_definitions,
};
pub use height::{
    AmplifyHeightStage, FlatHeightStage, HeightAmplifyMode, HeightNoiseMode, NoiseHeightStage,
};
pub use mesh::HeightFieldMeshStage;
pub use scattering::{PoissonScatterConfig, PoissonScatterStage};
