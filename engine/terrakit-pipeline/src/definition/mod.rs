//! Interface-facing stage definition and pipeline assembly APIs.
//!
//! This module describes stage types, parameter schemas, port schemas, resolved
//! port-resource bindings, and ordered construction requests. It intentionally
//! does not model visual nodes, graph links, node positions, graph persistence,
//! or topological sorting.
//!
//! Interfaces own their graph representation. To build an executable pipeline,
//! an interface assigns a [`crate::ResourceKey`] to each output socket, assigns
//! the same key to each connected input socket, supplies stages in execution
//! order, and submits those resolved values through
//! [`TerrainPipelineAssembler`]. Branching is represented by several input
//! bindings referring to one previously produced resource key.
//!
//! Saved interface nodes should store a [`StageTypeId`] and
//! [`StageSchemaVersion`] together. The schema version is incremented when a
//! schema or construction-semantics change can alter how existing saved node
//! data is interpreted. It is separate from the TerraKit crate version and the
//! future embedding C ABI version.

mod assembly;
mod error;
mod identifier;
mod parameter;
mod port;
mod registry;
mod schema;

pub use assembly::*;
pub use error::*;
pub use identifier::*;
pub use parameter::*;
pub use port::*;
pub use registry::*;
pub use schema::*;
