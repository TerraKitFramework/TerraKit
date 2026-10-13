//! Lightweight per-invocation generation requests.

use terrakit_core::{
    GenerationSeed, LodLevel, RegionCoord2, RegionCoord3, RegionRequest2, RegionRequest3,
};

use crate::GenerationRegionKind;

/// Lightweight request for one high-level TerraKit generation invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationRequest {
    /// Request one 2D region for the supplied seed, coordinate, and LOD tier.
    Region2 {
        /// Generation seed for this invocation.
        seed: GenerationSeed,

        /// Lightweight 2D region request resolved by the runtime layout.
        region: RegionRequest2,
    },

    /// Request one 3D region for the supplied seed, coordinate, and LOD tier.
    Region3 {
        /// Generation seed for this invocation.
        seed: GenerationSeed,

        /// Lightweight 3D region request resolved by the runtime layout.
        region: RegionRequest3,
    },
}

impl GenerationRequest {
    /// Creates a 2D generation request from the values that vary per invocation.
    pub const fn region2(seed: GenerationSeed, coordinate: RegionCoord2, lod: LodLevel) -> Self {
        Self::Region2 {
            seed,
            region: RegionRequest2::new(coordinate, lod),
        }
    }

    /// Creates a 3D generation request from the values that vary per invocation.
    pub const fn region3(seed: GenerationSeed, coordinate: RegionCoord3, lod: LodLevel) -> Self {
        Self::Region3 {
            seed,
            region: RegionRequest3::new(coordinate, lod),
        }
    }

    /// Returns the generation seed for this invocation.
    pub const fn seed(&self) -> GenerationSeed {
        match self {
            Self::Region2 { seed, .. } | Self::Region3 { seed, .. } => *seed,
        }
    }

    /// Returns the requested opaque LOD tier unchanged.
    pub const fn lod(&self) -> LodLevel {
        match self {
            Self::Region2 { region, .. } => region.lod,
            Self::Region3 { region, .. } => region.lod,
        }
    }

    /// Returns the dimensionality of this request.
    pub const fn kind(&self) -> GenerationRegionKind {
        match self {
            Self::Region2 { .. } => GenerationRegionKind::Region2,
            Self::Region3 { .. } => GenerationRegionKind::Region3,
        }
    }

    /// Returns the 2D region request when this request is two-dimensional.
    pub const fn as_region2(&self) -> Option<&RegionRequest2> {
        match self {
            Self::Region2 { region, .. } => Some(region),
            Self::Region3 { .. } => None,
        }
    }

    /// Returns the 3D region request when this request is three-dimensional.
    pub const fn as_region3(&self) -> Option<&RegionRequest3> {
        match self {
            Self::Region2 { .. } => None,
            Self::Region3 { region, .. } => Some(region),
        }
    }
}

#[cfg(test)]
mod tests {
    use terrakit_core::{RegionCoord2, RegionCoord3};

    use super::*;

    #[test]
    fn two_dimensional_constructor_preserves_seed_coordinate_and_lod() {
        let seed = GenerationSeed::new(123);
        let coordinate = RegionCoord2::new(4, -2);
        let lod = LodLevel::new(7);
        let request = GenerationRequest::region2(seed, coordinate, lod);

        assert_eq!(request.seed(), seed);
        assert_eq!(request.lod(), lod);
        assert_eq!(request.kind(), GenerationRegionKind::Region2);
        assert_eq!(
            request.as_region2(),
            Some(&RegionRequest2::new(coordinate, lod))
        );
        assert!(request.as_region3().is_none());
    }

    #[test]
    fn three_dimensional_constructor_preserves_seed_coordinate_and_lod() {
        let seed = GenerationSeed::new(456);
        let coordinate = RegionCoord3::new(-1, 2, 3);
        let lod = LodLevel::new(5);
        let request = GenerationRequest::region3(seed, coordinate, lod);

        assert_eq!(request.seed(), seed);
        assert_eq!(request.lod(), lod);
        assert_eq!(request.kind(), GenerationRegionKind::Region3);
        assert_eq!(
            request.as_region3(),
            Some(&RegionRequest3::new(coordinate, lod))
        );
        assert!(request.as_region2().is_none());
    }
}
