//! Executable terrain stage contracts.
//!
//! Stages declare resource requirements and products, then transform one
//! mutable resource set in serial pipeline order.

use crate::{ResourceKey, ResourceKind, ResourceSet, StageContext, StageError};

/// Stable numeric identifier for a terrain stage.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StageId(
    /// Raw stable stage identifier.
    pub u64,
);

/// Describes how a stage intends to access a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAccess {
    /// The stage reads the resource without modifying it.
    Read,

    /// The stage writes or creates the resource without relying on previous data.
    Write,

    /// The stage reads and mutates the resource.
    ReadWrite,
}

/// A resource that must be available before a stage executes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRequirement {
    /// Resource key required by the stage.
    pub key: ResourceKey,

    /// Expected kind for the resource at `key`.
    pub kind: ResourceKind,

    /// Declared access mode for this resource.
    pub access: ResourceAccess,

    /// Whether execution may continue when the resource is missing.
    pub optional: bool,
}

impl ResourceRequirement {
    /// Declares a mandatory resource requirement.
    pub const fn required(key: ResourceKey, kind: ResourceKind, access: ResourceAccess) -> Self {
        Self {
            key,
            kind,
            access,
            optional: false,
        }
    }

    /// Declares an optional resource requirement.
    pub const fn optional(key: ResourceKey, kind: ResourceKind, access: ResourceAccess) -> Self {
        Self {
            key,
            kind,
            access,
            optional: true,
        }
    }
}

/// A resource that a stage promises to produce after successful execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceProduct {
    /// Resource key produced by the stage.
    pub key: ResourceKey,

    /// Expected kind for the produced resource.
    pub kind: ResourceKind,

    /// Whether an existing resource may be replaced or modified.
    pub replace: bool,
}

impl ResourceProduct {
    /// Declares that a stage creates a resource at an empty key.
    pub const fn create(key: ResourceKey, kind: ResourceKind) -> Self {
        Self {
            key,
            kind,
            replace: false,
        }
    }

    /// Declares that a stage may replace or mutate a resource at `key`.
    pub const fn replace(key: ResourceKey, kind: ResourceKind) -> Self {
        Self {
            key,
            kind,
            replace: true,
        }
    }
}

/// Object-safe interface implemented by serial terrain transformation stages.
pub trait TerrainStage: Send {
    /// Returns the stable numeric stage identifier.
    fn id(&self) -> StageId;

    /// Returns a human-readable stage name for diagnostics.
    fn name(&self) -> &str;

    /// Returns declared input resources.
    fn requirements(&self) -> &[ResourceRequirement] {
        &[]
    }

    /// Returns declared output resources.
    fn products(&self) -> &[ResourceProduct] {
        &[]
    }

    /// Executes this stage against the mutable resource set.
    fn execute(
        &mut self,
        context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError>;
}
