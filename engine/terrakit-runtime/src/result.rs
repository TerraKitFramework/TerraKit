//! Owned results returned from successful runtime generation.

use terrakit_core::{
    GenerationRegion, GenerationSeed, LodLevel, RegionDescriptor2, RegionDescriptor3,
};
use terrakit_pipeline::ResourceSet;

use crate::{GenerationRegionKind, GenerationRequest};

/// Owned output for one completed high-level generation request.
///
/// A `GenerationResult` owns all resources produced for one generation request.
/// It does not borrow from its originating runtime and may outlive that runtime.
/// Generated resources are exposed immutably; a future ABI view may borrow
/// resource buffers until the owning result is destroyed.
#[derive(Debug)]
pub struct GenerationResult {
    request: GenerationRequest,
    region: GenerationRegion,
    resources: ResourceSet,
}

impl GenerationResult {
    pub(crate) fn new(
        request: GenerationRequest,
        region: GenerationRegion,
        resources: ResourceSet,
    ) -> Self {
        Self {
            request,
            region,
            resources,
        }
    }

    /// Returns the original lightweight generation request.
    pub const fn request(&self) -> GenerationRequest {
        self.request
    }

    /// Returns the generation seed for this result.
    pub const fn seed(&self) -> GenerationSeed {
        self.request.seed()
    }

    /// Returns the original requested opaque LOD tier.
    pub const fn lod(&self) -> LodLevel {
        self.request.lod()
    }

    /// Returns the resolved generation region used to execute the pipeline.
    pub const fn region(&self) -> &GenerationRegion {
        &self.region
    }

    /// Returns the dimensionality of this result.
    pub const fn kind(&self) -> GenerationRegionKind {
        self.request.kind()
    }

    /// Returns the owned resource set by shared reference.
    ///
    /// The result owns these resources and exposes them immutably. Borrowed
    /// resource views remain tied to this result's lifetime and do not depend
    /// on the originating runtime remaining alive.
    pub fn resources(&self) -> &ResourceSet {
        &self.resources
    }

    /// Consumes the result and returns its owned resource set.
    pub fn into_resources(self) -> ResourceSet {
        self.resources
    }

    /// Consumes the result and returns its request, resolved region, and resources.
    pub fn into_parts(self) -> (GenerationRequest, GenerationRegion, ResourceSet) {
        (self.request, self.region, self.resources)
    }

    /// Returns the 2D descriptor when this is a 2D result.
    pub fn region2(&self) -> Option<&RegionDescriptor2> {
        self.region.as_region2()
    }

    /// Returns the 3D descriptor when this is a 3D result.
    pub fn region3(&self) -> Option<&RegionDescriptor3> {
        self.region.as_region3()
    }
}
