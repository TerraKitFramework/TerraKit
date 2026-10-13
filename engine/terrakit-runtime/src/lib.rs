//! TerraKit's synchronous headless execution boundary.
//!
//! The runtime owns one configured generation layout and one configured terrain
//! pipeline. Callers submit lightweight [`GenerationRequest`] values containing
//! only seed, coordinate, and opaque LOD tier; the runtime resolves each request
//! through its layout, constructs a pipeline [`terrakit_pipeline::StageContext`],
//! creates a fresh per-run resource set, executes the pipeline, and returns an
//! independently owned [`GenerationResult`].
//!
//! A `GenerationResult` owns all resources produced for one generation request.
//! It does not borrow from its originating runtime and may outlive that runtime.
//!
//! Pipeline stage state belongs to the runtime and persists across calls.
//! Per-run resources are recreated for every call and become owned by the result
//! only after successful generation. Failed high-level generation does not expose
//! partially generated resources. Pipeline stage state and external side effects
//! are not rolled back.
//!
//! The runtime does not perform rendering, streaming, networking, caching,
//! serialization, or scheduling. One runtime processes one request at a time;
//! parallel hosts should create one runtime instance per worker thread or node.
//!
//! Future C handles are expected to map cleanly onto this ownership model:
//! `tk_runtime_t` owns a `TerrainRuntime`, while `tk_generation_result_t` owns a
//! `GenerationResult`. Buffer views should borrow from the result handle, not
//! the runtime handle, so an already-created result remains valid after the
//! runtime is destroyed.
//!
//! ```
//! # use terrakit_core::{
//! #     Extent2, GenerationSeed, LodLevel, RegionCoord2,
//! #     RegionLayout2, Vector2F64,
//! # };
//! # use terrakit_pipeline::TerrainPipeline;
//! # use terrakit_runtime::{
//! #     GenerationRequest, TerrainRuntime,
//! # };
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let layout = RegionLayout2::new(
//!     Extent2::try_new(32, 32)?,
//!     Vector2F64::new(1.0, 1.0),
//! )?;
//!
//! let pipeline = TerrainPipeline::new();
//! let mut runtime = TerrainRuntime::new(layout, pipeline);
//!
//! let result = runtime.generate(
//!     GenerationRequest::region2(
//!         GenerationSeed::new(1234),
//!         RegionCoord2::new(4, -2),
//!         LodLevel::HIGHEST,
//!     ),
//! )?;
//!
//! assert_eq!(result.seed(), GenerationSeed::new(1234));
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

/// Runtime error type.
mod error;
/// Runtime-owned generation layout enum and region-kind tag.
mod layout;
/// Lightweight generation request values.
mod request;
/// Owned generation result values.
mod result;
/// Synchronous runtime implementation.
mod runtime;

pub use error::*;
pub use layout::*;
pub use request::*;
pub use result::*;
pub use runtime::*;
