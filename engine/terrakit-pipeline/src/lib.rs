//! Serial terrain transformation pipelines for TerraKit.
//!
//! The pipeline crate owns stage execution and per-run resource storage. It
//! intentionally stays independent of engine wrappers, plugin loading,
//! scheduling, and asynchronous execution.
//!
//! The definition layer is a construction API for interface-owned node graphs:
//! an interface stores visual nodes, links, positions, graph persistence, and
//! undo state outside this crate. It compiles that graph into ordered
//! [`StageConstruction`] values with typed parameters and direct
//! port-to-[`ResourceKey`] bindings. [`TerrainPipelineAssembler`] validates the
//! resolved values and builds an ordinary [`TerrainPipeline`].
//!
//! A connection is represented by shared resource keys, not by an engine graph
//! edge. For example, a height output assigned `ResourceKey(100)` can feed two
//! downstream inputs by binding both of those input ports to `ResourceKey(100)`.
//! Interface-facing modifier stages read their inputs and create distinct
//! outputs, so branch results do not depend on execution order.
//!
//! [`StageTypeId`] identifies a stage kind such as `terrakit.height.noise`.
//! [`StageSchemaVersion`] identifies one compatible revision of that kind's
//! ports, parameters, defaults, constraints, and construction semantics.
//! [`StageId`] identifies one executable stage instance in an assembled
//! pipeline. Saved interface nodes should store both the stage type ID and the
//! schema version they were authored against.
//! `ResourceKey`, `StageId`, and `StageSchemaVersion` are transparent integer
//! wrappers suitable for layout regression tests, but complex pipeline values
//! such as `ResourceSet`, `StageSchema`, `ParameterValue`, and
//! `TerrainPipeline` should remain behind Rust APIs or future explicit ABI
//! handle/view structures.
//!
//! [`StageDefinition`] describes a discoverable stage type and creates
//! executable stages. [`StageConstruction`] is a transient resolved request to
//! create one configured stage. [`TerrainStage`] is the executable configured
//! stage instance. [`TerrainPipeline`] is the ordered executable stage list.

#![warn(missing_docs)]

/// Immutable per-execution context passed to stages.
mod context;
/// Interface-facing stage schemas, bindings, parameters, and assembly APIs.
mod definition;
/// Pipeline, stage, and assembly error types.
mod error;
/// Ordered serial terrain pipeline execution.
mod pipeline;
/// Per-execution resource storage and resource keys.
mod resource;
/// Stage identifiers, contracts, and executable stage trait.
mod stage;

pub use context::*;
pub use definition::*;
pub use error::*;
pub use pipeline::*;
pub use resource::*;
pub use stage::*;
