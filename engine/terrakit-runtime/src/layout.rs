//! Runtime-owned generation layout wrapper.

use terrakit_core::{RegionLayout2, RegionLayout3};

/// Runtime-level dimensionality identifier for generation layouts and requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationRegionKind {
    /// Two-dimensional region generation.
    Region2,

    /// Three-dimensional region generation.
    Region3,
}

/// Configured immutable generation layout owned by a [`crate::TerrainRuntime`].
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationLayout {
    /// A 2D region layout.
    Region2(RegionLayout2),

    /// A 3D region layout.
    Region3(RegionLayout3),
}

impl From<RegionLayout2> for GenerationLayout {
    fn from(layout: RegionLayout2) -> Self {
        Self::Region2(layout)
    }
}

impl From<RegionLayout3> for GenerationLayout {
    fn from(layout: RegionLayout3) -> Self {
        Self::Region3(layout)
    }
}

impl GenerationLayout {
    /// Returns the dimensionality of this configured layout.
    pub const fn kind(&self) -> GenerationRegionKind {
        match self {
            Self::Region2(_) => GenerationRegionKind::Region2,
            Self::Region3(_) => GenerationRegionKind::Region3,
        }
    }

    /// Returns the 2D layout when this configuration is two-dimensional.
    pub const fn as_region2(&self) -> Option<&RegionLayout2> {
        match self {
            Self::Region2(layout) => Some(layout),
            Self::Region3(_) => None,
        }
    }

    /// Returns the 3D layout when this configuration is three-dimensional.
    pub const fn as_region3(&self) -> Option<&RegionLayout3> {
        match self {
            Self::Region2(_) => None,
            Self::Region3(layout) => Some(layout),
        }
    }
}

#[cfg(test)]
mod tests {
    use terrakit_core::{Extent2, Extent3, Vector2F64, Vector3F64};

    use super::*;

    fn layout2() -> RegionLayout2 {
        RegionLayout2::new(Extent2::try_new(8, 4).unwrap(), Vector2F64::new(1.0, 2.0)).unwrap()
    }

    fn layout3() -> RegionLayout3 {
        RegionLayout3::new(
            Extent3::try_new(8, 4, 2).unwrap(),
            Vector3F64::new(1.0, 2.0, 3.0),
        )
        .unwrap()
    }

    #[test]
    fn two_dimensional_core_layout_converts_into_generation_layout() {
        let layout = layout2();
        let generation_layout = GenerationLayout::from(layout);

        assert_eq!(generation_layout, GenerationLayout::Region2(layout));
    }

    #[test]
    fn three_dimensional_core_layout_converts_into_generation_layout() {
        let layout = layout3();
        let generation_layout = GenerationLayout::from(layout);

        assert_eq!(generation_layout, GenerationLayout::Region3(layout));
    }

    #[test]
    fn kind_reports_configured_dimensionality() {
        assert_eq!(
            GenerationLayout::from(layout2()).kind(),
            GenerationRegionKind::Region2
        );
        assert_eq!(
            GenerationLayout::from(layout3()).kind(),
            GenerationRegionKind::Region3
        );
    }

    #[test]
    fn typed_accessors_are_dimension_safe() {
        let layout2 = GenerationLayout::from(layout2());
        let layout3 = GenerationLayout::from(layout3());

        assert!(layout2.as_region2().is_some());
        assert!(layout2.as_region3().is_none());
        assert!(layout3.as_region3().is_some());
        assert!(layout3.as_region2().is_none());
    }
}
