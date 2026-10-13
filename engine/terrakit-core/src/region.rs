//! Canonical region addressing and layout resolution.
//!
//! A TerraKit region is the engine-independent unit of terrain generation. In
//! chunked game worlds a region will commonly map to one engine chunk, but the
//! core model deliberately avoids engine streaming, chunk ownership, networking,
//! and persistence concerns.
//!
//! Region layouts define the shared cell size, spacing, and alignment rules for
//! a world or generation service. Region requests identify one coordinate and
//! an opaque generation-detail tier within that layout; they do not carry
//! arbitrary origins, dimensions, or sample divisors. This keeps all regions
//! generated under the same layout aligned to identical base cell extents,
//! spacing rules, and coordinate conventions.
//!
//! Cells describe areas or volumes. Point samples live on grid points or cell
//! corners, so a 32 by 32 cell region has a 33 by 33 point-sample extent.
//! Level zero conventionally represents the highest configured detail. The
//! current built-in region-layout policy maps increasing levels to
//! power-of-two spatial reduction while preserving fixed world coverage:
//! resolved cell resolution decreases and effective spacing increases. The
//! [`LodLevel`] value itself remains an opaque detail-tier identifier so future
//! world, pipeline, or stage policies can interpret configured levels
//! differently without changing request or descriptor types.
//!
//! Stages receive the original requested tier through their stage context and
//! may eventually map that same tier to algorithm-specific quality settings.
//! Engine rendering LOD is separate from TerraKit generation LOD: an engine or
//! game server may choose which [`LodLevel`] to request, while rendering
//! distances, culling, transitions, and mesh selection remain engine concerns.
//!
//! Two-dimensional regions use TerraKit's standard horizontal XZ placement:
//! region coordinate X advances world X, and region coordinate Y advances world
//! Z. Three-dimensional regions use standard XYZ placement. Adjacent
//! point-sampled regions therefore calculate the same world-space positions on
//! shared boundaries when generated independently.
//!
//! Extremely large integer coordinates remain valid addresses, but their
//! resolved `f64` world positions may eventually lose positional precision.
//! Non-finite resolved origins are rejected.
//!
//! ```
//! # use terrakit_core::{
//! #     CoreError, Extent2, LodLevel, RegionCoord2, RegionLayout2, RegionRequest2, Vector2F64,
//! # };
//! # fn example() -> Result<(), CoreError> {
//! let layout = RegionLayout2::new(
//!     Extent2::try_new(32, 32)?,
//!     Vector2F64::new(1.0, 1.0),
//! )?;
//!
//! let requested_lod = LodLevel::new(2);
//! let descriptor = layout.resolve(RegionRequest2::new(
//!     RegionCoord2::new(4, -2),
//!     requested_lod,
//! ))?;
//!
//! assert_eq!(descriptor.lod(), requested_lod);
//! # Ok(())
//! # }
//! ```

use crate::{
    error::CoreError,
    grid::{Extent2, Extent3, GridTransform2, GridTransform3},
    math::{Vector2F64, Vector3F64},
};

/// Integer address of a 2D region within a specific [`RegionLayout2`].
///
/// Coordinates are not world-space units, and negative coordinates are valid.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RegionCoord2 {
    /// Horizontal region address. In standard 2D placement this advances world X.
    pub x: i64,

    /// Second region address. In standard 2D placement this advances world Z.
    pub y: i64,
}

impl RegionCoord2 {
    /// The zero coordinate.
    pub const ZERO: Self = Self { x: 0, y: 0 };

    /// Creates a 2D region coordinate.
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

/// Integer address of a 3D region within a specific [`RegionLayout3`].
///
/// Coordinates are not world-space units, and negative coordinates are valid.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RegionCoord3 {
    /// Region address along world X.
    pub x: i64,

    /// Region address along world Y.
    pub y: i64,

    /// Region address along world Z.
    pub z: i64,
}

impl RegionCoord3 {
    /// The zero coordinate.
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    /// Creates a 3D region coordinate.
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }
}

/// Identifies a requested generation-detail tier.
///
/// Level zero conventionally represents the highest configured detail.
/// Increasing values conventionally identify progressively lower-detail tiers.
///
/// The numeric value is an opaque selector. It does not intrinsically define
/// a spatial scale, resolution divisor, rendering distance, or algorithm
/// quality. Layouts and stages may interpret configured levels according to
/// their own policies.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LodLevel(
    /// Raw opaque detail-tier selector.
    pub u16,
);

impl LodLevel {
    /// Conventional highest configured detail tier.
    pub const HIGHEST: Self = Self(0);

    /// Creates a detail-tier identifier.
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the opaque numeric tier selector.
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Shared size, spacing, alignment, and default sampling policy for 2D regions.
///
/// All regions generated with one layout use identical base cell extents,
/// spacing rules, and coordinate alignment. The current default spatial policy
/// maps requested detail tiers to power-of-two spatial reduction while
/// preserving fixed world coverage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionLayout2 {
    cell_extent: Extent2,
    base_spacing: Vector2F64,
}

impl RegionLayout2 {
    /// Creates a validated 2D region layout.
    pub fn new(cell_extent: Extent2, base_spacing: Vector2F64) -> Result<Self, CoreError> {
        validate_spacing_2(base_spacing)?;

        let layout = Self {
            cell_extent,
            base_spacing,
        };
        layout.world_size()?;

        Ok(layout)
    }

    /// Returns the base cell extent shared by all regions in this layout.
    pub const fn cell_extent(&self) -> Extent2 {
        self.cell_extent
    }

    /// Returns the conventional highest-detail cell spacing.
    pub const fn base_spacing(&self) -> Vector2F64 {
        self.base_spacing
    }

    /// Returns the cell-sampled extent resolved for `lod` by the current layout policy.
    pub fn cell_sample_extent(&self, lod: LodLevel) -> Result<Extent2, CoreError> {
        let scale = default_spatial_scale(lod)?;
        let width = lod_cell_count("x", self.cell_extent.width(), scale, lod)?;
        let height = lod_cell_count("y", self.cell_extent.height(), scale, lod)?;

        Extent2::try_new(width, height)
    }

    /// Returns the point-sampled extent for `lod`.
    ///
    /// Point samples include shared region boundaries, so each axis is the
    /// policy-resolved cell count plus one.
    pub fn point_sample_extent(&self, lod: LodLevel) -> Result<Extent2, CoreError> {
        let cell_extent = self.cell_sample_extent(lod)?;
        let width = point_sample_axis(cell_extent.width())?;
        let height = point_sample_axis(cell_extent.height())?;

        Extent2::try_new(width, height)
    }

    /// Returns the effective point spacing resolved for `lod` by the current layout policy.
    pub fn effective_spacing(&self, lod: LodLevel) -> Result<Vector2F64, CoreError> {
        let scale = default_spatial_scale(lod)? as f64;

        Ok(Vector2F64::new(
            scaled_spacing("x", self.base_spacing.x, scale)?,
            scaled_spacing("y", self.base_spacing.y, scale)?,
        ))
    }

    /// Returns the fixed world-space size covered by every region in this layout.
    pub fn world_size(&self) -> Result<Vector2F64, CoreError> {
        Ok(Vector2F64::new(
            scaled_coverage("x", self.cell_extent.width(), self.base_spacing.x)?,
            scaled_coverage("y", self.cell_extent.height(), self.base_spacing.y)?,
        ))
    }

    /// Resolves a lightweight request into concrete generation data.
    ///
    /// The returned descriptor preserves the requested detail tier unchanged
    /// alongside spatial data resolved by this layout's current policy.
    pub fn resolve(&self, request: RegionRequest2) -> Result<RegionDescriptor2, CoreError> {
        let cell_extent = self.cell_sample_extent(request.lod)?;
        let point_sample_extent = self.point_sample_extent(request.lod)?;
        let effective_spacing = self.effective_spacing(request.lod)?;
        let world_size = self.world_size()?;
        let origin_x = region_origin_axis(request.coordinate.x, world_size.x)?;
        let origin_z = region_origin_axis(request.coordinate.y, world_size.y)?;
        let transform = GridTransform2::from_axes(
            Vector3F64::new(origin_x, 0.0, origin_z),
            Vector3F64::new(effective_spacing.x, 0.0, 0.0),
            Vector3F64::new(0.0, 0.0, effective_spacing.y),
        )?;

        Ok(RegionDescriptor2 {
            coordinate: request.coordinate,
            lod: request.lod,
            cell_extent,
            point_sample_extent,
            effective_spacing,
            transform,
        })
    }
}

/// Shared size, spacing, alignment, and default sampling policy for 3D regions.
///
/// All regions generated with one layout use identical base cell extents,
/// spacing rules, and coordinate alignment. The current default spatial policy
/// maps requested detail tiers to power-of-two spatial reduction while
/// preserving fixed world coverage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionLayout3 {
    cell_extent: Extent3,
    base_spacing: Vector3F64,
}

impl RegionLayout3 {
    /// Creates a validated 3D region layout.
    pub fn new(cell_extent: Extent3, base_spacing: Vector3F64) -> Result<Self, CoreError> {
        validate_spacing_3(base_spacing)?;

        let layout = Self {
            cell_extent,
            base_spacing,
        };
        layout.world_size()?;

        Ok(layout)
    }

    /// Returns the base cell extent shared by all regions in this layout.
    pub const fn cell_extent(&self) -> Extent3 {
        self.cell_extent
    }

    /// Returns the conventional highest-detail cell spacing.
    pub const fn base_spacing(&self) -> Vector3F64 {
        self.base_spacing
    }

    /// Returns the cell-sampled extent resolved for `lod` by the current layout policy.
    pub fn cell_sample_extent(&self, lod: LodLevel) -> Result<Extent3, CoreError> {
        let scale = default_spatial_scale(lod)?;
        let width = lod_cell_count("x", self.cell_extent.width(), scale, lod)?;
        let height = lod_cell_count("y", self.cell_extent.height(), scale, lod)?;
        let depth = lod_cell_count("z", self.cell_extent.depth(), scale, lod)?;

        Extent3::try_new(width, height, depth)
    }

    /// Returns the point-sampled extent for `lod`.
    ///
    /// Point samples include shared region boundaries, so each axis is the
    /// policy-resolved cell count plus one.
    pub fn point_sample_extent(&self, lod: LodLevel) -> Result<Extent3, CoreError> {
        let cell_extent = self.cell_sample_extent(lod)?;
        let width = point_sample_axis(cell_extent.width())?;
        let height = point_sample_axis(cell_extent.height())?;
        let depth = point_sample_axis(cell_extent.depth())?;

        Extent3::try_new(width, height, depth)
    }

    /// Returns the effective point spacing resolved for `lod` by the current layout policy.
    pub fn effective_spacing(&self, lod: LodLevel) -> Result<Vector3F64, CoreError> {
        let scale = default_spatial_scale(lod)? as f64;

        Ok(Vector3F64::new(
            scaled_spacing("x", self.base_spacing.x, scale)?,
            scaled_spacing("y", self.base_spacing.y, scale)?,
            scaled_spacing("z", self.base_spacing.z, scale)?,
        ))
    }

    /// Returns the fixed world-space size covered by every region in this layout.
    pub fn world_size(&self) -> Result<Vector3F64, CoreError> {
        Ok(Vector3F64::new(
            scaled_coverage("x", self.cell_extent.width(), self.base_spacing.x)?,
            scaled_coverage("y", self.cell_extent.height(), self.base_spacing.y)?,
            scaled_coverage("z", self.cell_extent.depth(), self.base_spacing.z)?,
        ))
    }

    /// Resolves a lightweight request into concrete generation data.
    ///
    /// The returned descriptor preserves the requested detail tier unchanged
    /// alongside spatial data resolved by this layout's current policy.
    pub fn resolve(&self, request: RegionRequest3) -> Result<RegionDescriptor3, CoreError> {
        let cell_extent = self.cell_sample_extent(request.lod)?;
        let point_sample_extent = self.point_sample_extent(request.lod)?;
        let effective_spacing = self.effective_spacing(request.lod)?;
        let world_size = self.world_size()?;
        let origin_x = region_origin_axis(request.coordinate.x, world_size.x)?;
        let origin_y = region_origin_axis(request.coordinate.y, world_size.y)?;
        let origin_z = region_origin_axis(request.coordinate.z, world_size.z)?;
        let transform = GridTransform3::from_axes(
            Vector3F64::new(origin_x, origin_y, origin_z),
            Vector3F64::new(effective_spacing.x, 0.0, 0.0),
            Vector3F64::new(0.0, effective_spacing.y, 0.0),
            Vector3F64::new(0.0, 0.0, effective_spacing.z),
        )?;

        Ok(RegionDescriptor3 {
            coordinate: request.coordinate,
            lod: request.lod,
            cell_extent,
            point_sample_extent,
            effective_spacing,
            transform,
        })
    }
}

/// Lightweight request for one 2D region at one requested detail tier.
///
/// A request identifies where generation occurs and which configured detail
/// tier should be used. It does not define the implementation of that tier,
/// provide a sample divisor, or carry arbitrary dimensions. [`RegionLayout2`]
/// currently resolves the selected tier into concrete spatial sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionRequest2 {
    /// Region coordinate within a layout.
    pub coordinate: RegionCoord2,

    /// Selected opaque generation-detail tier.
    pub lod: LodLevel,
}

impl RegionRequest2 {
    /// Creates a 2D region request.
    pub const fn new(coordinate: RegionCoord2, lod: LodLevel) -> Self {
        Self { coordinate, lod }
    }
}

/// Lightweight request for one 3D region at one requested detail tier.
///
/// A request identifies where generation occurs and which configured detail
/// tier should be used. It does not define the implementation of that tier,
/// provide a sample divisor, or carry arbitrary dimensions. [`RegionLayout3`]
/// currently resolves the selected tier into concrete spatial sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionRequest3 {
    /// Region coordinate within a layout.
    pub coordinate: RegionCoord3,

    /// Selected opaque generation-detail tier.
    pub lod: LodLevel,
}

impl RegionRequest3 {
    /// Creates a 3D region request.
    pub const fn new(coordinate: RegionCoord3, lod: LodLevel) -> Self {
        Self { coordinate, lod }
    }
}

/// Resolved generation data for a 2D region.
///
/// Descriptors retain the original requested detail tier alongside concrete
/// spatial results such as cell extent, point-sample extent, effective spacing,
/// and transform. Stages should read [`Self::lod`] instead of inferring the
/// requested tier from resolved sample dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionDescriptor2 {
    coordinate: RegionCoord2,
    lod: LodLevel,
    cell_extent: Extent2,
    point_sample_extent: Extent2,
    effective_spacing: Vector2F64,
    transform: GridTransform2,
}

impl RegionDescriptor2 {
    /// Returns the region coordinate.
    pub const fn coordinate(&self) -> RegionCoord2 {
        self.coordinate
    }

    /// Returns the original requested generation-detail tier.
    pub const fn lod(&self) -> LodLevel {
        self.lod
    }

    /// Returns the concrete cell extent resolved by the current layout policy.
    pub const fn cell_extent(&self) -> Extent2 {
        self.cell_extent
    }

    /// Returns the concrete point-sample extent resolved by the current layout policy.
    pub const fn point_sample_extent(&self) -> Extent2 {
        self.point_sample_extent
    }

    /// Returns the effective point spacing resolved by the current layout policy.
    pub const fn effective_spacing(&self) -> Vector2F64 {
        self.effective_spacing
    }

    /// Returns the world-space transform for this region.
    pub const fn transform(&self) -> GridTransform2 {
        self.transform
    }
}

/// Resolved generation data for a 3D region.
///
/// Descriptors retain the original requested detail tier alongside concrete
/// spatial results such as cell extent, point-sample extent, effective spacing,
/// and transform. Stages should read [`Self::lod`] instead of inferring the
/// requested tier from resolved sample dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionDescriptor3 {
    coordinate: RegionCoord3,
    lod: LodLevel,
    cell_extent: Extent3,
    point_sample_extent: Extent3,
    effective_spacing: Vector3F64,
    transform: GridTransform3,
}

impl RegionDescriptor3 {
    /// Returns the region coordinate.
    pub const fn coordinate(&self) -> RegionCoord3 {
        self.coordinate
    }

    /// Returns the original requested generation-detail tier.
    pub const fn lod(&self) -> LodLevel {
        self.lod
    }

    /// Returns the concrete cell extent resolved by the current layout policy.
    pub const fn cell_extent(&self) -> Extent3 {
        self.cell_extent
    }

    /// Returns the concrete point-sample extent resolved by the current layout policy.
    pub const fn point_sample_extent(&self) -> Extent3 {
        self.point_sample_extent
    }

    /// Returns the effective point spacing resolved by the current layout policy.
    pub const fn effective_spacing(&self) -> Vector3F64 {
        self.effective_spacing
    }

    /// Returns the world-space transform for this region.
    pub const fn transform(&self) -> GridTransform3 {
        self.transform
    }
}

/// Dimension-tagged resolved region data for pipeline execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenerationRegion {
    /// A resolved 2D generation region.
    Region2(RegionDescriptor2),

    /// A resolved 3D generation region.
    Region3(RegionDescriptor3),
}

impl GenerationRegion {
    /// Returns the requested generation-detail level.
    pub const fn lod(&self) -> LodLevel {
        match self {
            Self::Region2(region) => region.lod(),
            Self::Region3(region) => region.lod(),
        }
    }

    /// Returns the 2D descriptor when this is a 2D region.
    pub fn as_region2(&self) -> Option<&RegionDescriptor2> {
        match self {
            Self::Region2(region) => Some(region),
            Self::Region3(_) => None,
        }
    }

    /// Returns the 3D descriptor when this is a 3D region.
    pub fn as_region3(&self) -> Option<&RegionDescriptor3> {
        match self {
            Self::Region2(_) => None,
            Self::Region3(region) => Some(region),
        }
    }
}

fn validate_spacing_2(spacing: Vector2F64) -> Result<(), CoreError> {
    validate_spacing_axis("x", spacing.x)?;
    validate_spacing_axis("y", spacing.y)
}

fn validate_spacing_3(spacing: Vector3F64) -> Result<(), CoreError> {
    validate_spacing_axis("x", spacing.x)?;
    validate_spacing_axis("y", spacing.y)?;
    validate_spacing_axis("z", spacing.z)
}

fn validate_spacing_axis(axis: &'static str, spacing: f64) -> Result<(), CoreError> {
    if !spacing.is_finite() || spacing <= 0.0 {
        return Err(CoreError::InvalidSpacing { axis });
    }

    Ok(())
}

fn default_spatial_scale(lod: LodLevel) -> Result<u64, CoreError> {
    1_u64
        .checked_shl(u32::from(lod.value()))
        .ok_or(CoreError::LodScaleOverflow { lod })
}

fn lod_cell_count(
    axis: &'static str,
    cell_count: u32,
    scale: u64,
    lod: LodLevel,
) -> Result<u32, CoreError> {
    if scale > u64::from(u32::MAX) {
        return Err(CoreError::UnsupportedLod { lod });
    }

    let cell_count_u64 = u64::from(cell_count);
    if cell_count_u64 % scale != 0 {
        return Err(CoreError::LodNotDivisible {
            axis,
            cell_count,
            scale,
        });
    }

    let reduced = cell_count_u64 / scale;
    if reduced == 0 {
        return Err(CoreError::UnsupportedLod { lod });
    }

    u32::try_from(reduced).map_err(|_| CoreError::DimensionOverflow)
}

fn point_sample_axis(cell_count: u32) -> Result<u32, CoreError> {
    cell_count
        .checked_add(1)
        .ok_or(CoreError::DimensionOverflow)
}

fn scaled_spacing(axis: &'static str, spacing: f64, scale: f64) -> Result<f64, CoreError> {
    let scaled = spacing * scale;

    if !scaled.is_finite() || scaled <= 0.0 {
        return Err(CoreError::InvalidSpacing { axis });
    }

    Ok(scaled)
}

fn scaled_coverage(axis: &'static str, cell_count: u32, spacing: f64) -> Result<f64, CoreError> {
    let coverage = f64::from(cell_count) * spacing;

    if !coverage.is_finite() || coverage <= 0.0 {
        return Err(CoreError::InvalidSpacing { axis });
    }

    Ok(coverage)
}

fn region_origin_axis(coordinate: i64, world_size: f64) -> Result<f64, CoreError> {
    let origin = coordinate as f64 * world_size;

    if !origin.is_finite() {
        return Err(CoreError::NonFiniteRegionOrigin);
    }

    Ok(origin)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn layout2() -> RegionLayout2 {
        RegionLayout2::new(Extent2::try_new(32, 16).unwrap(), Vector2F64::new(1.5, 2.0)).unwrap()
    }

    fn layout3() -> RegionLayout3 {
        RegionLayout3::new(
            Extent3::try_new(16, 32, 8).unwrap(),
            Vector3F64::new(1.0, 2.0, 0.5),
        )
        .unwrap()
    }

    #[test]
    fn region_coordinates_construct_compare_hash_and_support_negative_values() {
        let coord2 = RegionCoord2::new(-4, 7);
        let coord3 = RegionCoord3::new(-1, 0, 2);
        let mut coords2 = HashSet::new();
        let mut coords3 = HashSet::new();

        coords2.insert(coord2);
        coords3.insert(coord3);

        assert_eq!(RegionCoord2::ZERO, RegionCoord2::new(0, 0));
        assert_eq!(RegionCoord3::ZERO, RegionCoord3::new(0, 0, 0));
        assert!(coords2.contains(&RegionCoord2::new(-4, 7)));
        assert!(coords3.contains(&RegionCoord3::new(-1, 0, 2)));
        assert_ne!(coord2, RegionCoord2::new(7, -4));
    }

    #[test]
    fn lod_level_is_opaque_identifier_with_stable_value_and_ordering() {
        assert_eq!(LodLevel::HIGHEST.value(), 0);
        assert_eq!(LodLevel::new(3).value(), 3);
        assert_eq!(LodLevel::new(2), LodLevel::new(2));
        assert!(LodLevel::new(1) > LodLevel::HIGHEST);
        assert!(LodLevel::new(2) > LodLevel::new(1));
    }

    #[test]
    fn region_requests_retain_selected_detail_tier() {
        let lod = LodLevel::new(7);
        let request2 = RegionRequest2::new(RegionCoord2::new(4, -2), lod);
        let request3 = RegionRequest3::new(RegionCoord3::new(-1, 2, 3), lod);

        assert_eq!(request2.lod, lod);
        assert_eq!(request3.lod, lod);
    }

    #[test]
    fn layout_validation_accepts_valid_2d_and_3d_layouts() {
        assert_eq!(layout2().cell_extent(), Extent2::try_new(32, 16).unwrap());
        assert_eq!(
            layout3().cell_extent(),
            Extent3::try_new(16, 32, 8).unwrap()
        );
    }

    #[test]
    fn layout_validation_rejects_zero_dimensions_before_layout_construction() {
        assert_eq!(Extent2::try_new(0, 32), Err(CoreError::ZeroDimension));
        assert_eq!(Extent3::try_new(16, 0, 16), Err(CoreError::ZeroDimension));
    }

    #[test]
    fn layout_validation_rejects_invalid_2d_spacing() {
        let extent = Extent2::try_new(32, 32).unwrap();

        assert_eq!(
            RegionLayout2::new(extent, Vector2F64::new(0.0, 1.0)),
            Err(CoreError::InvalidSpacing { axis: "x" })
        );
        assert_eq!(
            RegionLayout2::new(extent, Vector2F64::new(1.0, -1.0)),
            Err(CoreError::InvalidSpacing { axis: "y" })
        );
        assert_eq!(
            RegionLayout2::new(extent, Vector2F64::new(f64::INFINITY, 1.0)),
            Err(CoreError::InvalidSpacing { axis: "x" })
        );
        assert_eq!(
            RegionLayout2::new(extent, Vector2F64::new(1.0, f64::NAN)),
            Err(CoreError::InvalidSpacing { axis: "y" })
        );
    }

    #[test]
    fn layout_validation_rejects_invalid_3d_spacing() {
        let extent = Extent3::try_new(16, 16, 16).unwrap();

        assert_eq!(
            RegionLayout3::new(extent, Vector3F64::new(1.0, 0.0, 1.0)),
            Err(CoreError::InvalidSpacing { axis: "y" })
        );
        assert_eq!(
            RegionLayout3::new(extent, Vector3F64::new(1.0, 1.0, -0.25)),
            Err(CoreError::InvalidSpacing { axis: "z" })
        );
        assert_eq!(
            RegionLayout3::new(extent, Vector3F64::new(1.0, f64::INFINITY, 1.0)),
            Err(CoreError::InvalidSpacing { axis: "y" })
        );
        assert_eq!(
            RegionLayout3::new(extent, Vector3F64::new(1.0, 1.0, f64::NAN)),
            Err(CoreError::InvalidSpacing { axis: "z" })
        );
    }

    #[test]
    fn layout_validation_rejects_spacing_that_makes_world_coverage_non_finite() {
        let extent = Extent2::try_new(2, 2).unwrap();

        assert_eq!(
            RegionLayout2::new(extent, Vector2F64::new(f64::MAX, 1.0)),
            Err(CoreError::InvalidSpacing { axis: "x" })
        );
    }

    #[test]
    fn world_size_is_fixed_for_layout_and_independent_of_selected_level() {
        let layout2 = layout2();
        let layout3 = layout3();

        assert_eq!(layout2.world_size().unwrap(), Vector2F64::new(48.0, 32.0));
        assert_eq!(
            layout3.world_size().unwrap(),
            Vector3F64::new(16.0, 64.0, 4.0)
        );
        assert_eq!(
            layout2
                .resolve(request2(0, 0, 0))
                .unwrap()
                .transform()
                .origin()
                .x,
            0.0
        );
        assert_eq!(
            layout2
                .resolve(request2(0, 0, 2))
                .unwrap()
                .transform()
                .origin()
                .x,
            0.0
        );
        assert_eq!(
            layout3
                .resolve(request3(0, 0, 0, 0))
                .unwrap()
                .transform()
                .origin()
                .y,
            0.0
        );
        assert_eq!(
            layout3
                .resolve(request3(0, 0, 0, 2))
                .unwrap()
                .transform()
                .origin()
                .y,
            0.0
        );
    }

    #[test]
    fn default_spatial_policy_reduces_resolution_by_powers_of_two() {
        let layout =
            RegionLayout2::new(Extent2::try_new(32, 32).unwrap(), Vector2F64::new(1.0, 2.0))
                .unwrap();

        assert_eq!(
            layout.cell_sample_extent(LodLevel::HIGHEST).unwrap(),
            Extent2::try_new(32, 32).unwrap()
        );
        assert_eq!(
            layout.point_sample_extent(LodLevel::HIGHEST).unwrap(),
            Extent2::try_new(33, 33).unwrap()
        );
        assert_eq!(
            layout.cell_sample_extent(LodLevel::new(1)).unwrap(),
            Extent2::try_new(16, 16).unwrap()
        );
        assert_eq!(
            layout.point_sample_extent(LodLevel::new(1)).unwrap(),
            Extent2::try_new(17, 17).unwrap()
        );
        assert_eq!(
            layout.cell_sample_extent(LodLevel::new(2)).unwrap(),
            Extent2::try_new(8, 8).unwrap()
        );
        assert_eq!(
            layout.point_sample_extent(LodLevel::new(2)).unwrap(),
            Extent2::try_new(9, 9).unwrap()
        );
        assert_eq!(
            layout.effective_spacing(LodLevel::new(2)).unwrap(),
            Vector2F64::new(4.0, 8.0)
        );
    }

    #[test]
    fn default_spatial_policy_resolves_3d_layouts() {
        let layout = RegionLayout3::new(
            Extent3::try_new(32, 16, 8).unwrap(),
            Vector3F64::new(1.0, 1.0, 1.0),
        )
        .unwrap();

        assert_eq!(
            layout.cell_sample_extent(LodLevel::new(1)).unwrap(),
            Extent3::try_new(16, 8, 4).unwrap()
        );
        assert_eq!(
            layout.point_sample_extent(LodLevel::new(1)).unwrap(),
            Extent3::try_new(17, 9, 5).unwrap()
        );
        assert_eq!(
            layout.effective_spacing(LodLevel::new(1)).unwrap(),
            Vector3F64::new(2.0, 2.0, 2.0)
        );
    }

    #[test]
    fn default_spatial_policy_rejects_non_divisible_extents_and_excessive_levels() {
        let layout =
            RegionLayout2::new(Extent2::try_new(30, 32).unwrap(), Vector2F64::new(1.0, 1.0))
                .unwrap();

        assert_eq!(
            layout.cell_sample_extent(LodLevel::new(3)),
            Err(CoreError::LodNotDivisible {
                axis: "x",
                cell_count: 30,
                scale: 8,
            })
        );
        assert_eq!(
            layout.cell_sample_extent(LodLevel::new(32)),
            Err(CoreError::UnsupportedLod {
                lod: LodLevel::new(32)
            })
        );
        assert_eq!(
            layout.cell_sample_extent(LodLevel::new(64)),
            Err(CoreError::LodScaleOverflow {
                lod: LodLevel::new(64)
            })
        );
    }

    #[test]
    fn point_sample_extent_rejects_axis_overflow() {
        let layout = RegionLayout2::new(
            Extent2::try_new(u32::MAX, 1).unwrap(),
            Vector2F64::new(1.0, 1.0),
        )
        .unwrap();

        assert_eq!(
            layout.point_sample_extent(LodLevel::HIGHEST),
            Err(CoreError::DimensionOverflow)
        );
    }

    #[test]
    fn resolves_positive_and_negative_2d_origins_and_transforms() {
        let layout = layout2();
        let positive = layout.resolve(request2(2, 3, 0)).unwrap();
        let negative = layout.resolve(request2(-1, -2, 1)).unwrap();

        assert_eq!(positive.coordinate(), RegionCoord2::new(2, 3));
        assert_eq!(positive.lod(), LodLevel::HIGHEST);
        assert_eq!(positive.cell_extent(), Extent2::try_new(32, 16).unwrap());
        assert_eq!(
            positive.point_sample_extent(),
            Extent2::try_new(33, 17).unwrap()
        );
        assert_eq!(positive.effective_spacing(), Vector2F64::new(1.5, 2.0));
        assert_eq!(
            positive.transform().origin(),
            Vector3F64::new(96.0, 0.0, 96.0)
        );
        assert_eq!(
            positive.transform().axis_x(),
            Vector3F64::new(1.5, 0.0, 0.0)
        );
        assert_eq!(
            positive.transform().axis_y(),
            Vector3F64::new(0.0, 0.0, 2.0)
        );

        assert_eq!(negative.cell_extent(), Extent2::try_new(16, 8).unwrap());
        assert_eq!(
            negative.point_sample_extent(),
            Extent2::try_new(17, 9).unwrap()
        );
        assert_eq!(negative.effective_spacing(), Vector2F64::new(3.0, 4.0));
        assert_eq!(
            negative.transform().origin(),
            Vector3F64::new(-48.0, 0.0, -64.0)
        );
    }

    #[test]
    fn resolves_positive_and_negative_3d_origins_and_transforms() {
        let layout = layout3();
        let positive = layout.resolve(request3(2, 1, 3, 0)).unwrap();
        let negative = layout.resolve(request3(-1, -2, -3, 1)).unwrap();

        assert_eq!(
            positive.transform().origin(),
            Vector3F64::new(32.0, 64.0, 12.0)
        );
        assert_eq!(
            positive.transform().axis_x(),
            Vector3F64::new(1.0, 0.0, 0.0)
        );
        assert_eq!(
            positive.transform().axis_y(),
            Vector3F64::new(0.0, 2.0, 0.0)
        );
        assert_eq!(
            positive.transform().axis_z(),
            Vector3F64::new(0.0, 0.0, 0.5)
        );
        assert_eq!(negative.cell_extent(), Extent3::try_new(8, 16, 4).unwrap());
        assert_eq!(
            negative.point_sample_extent(),
            Extent3::try_new(9, 17, 5).unwrap()
        );
        assert_eq!(negative.effective_spacing(), Vector3F64::new(2.0, 4.0, 1.0));
        assert_eq!(
            negative.transform().origin(),
            Vector3F64::new(-16.0, -128.0, -12.0)
        );
    }

    #[test]
    fn same_coordinate_produces_identical_descriptors() {
        let layout = layout2();
        let request = request2(4, -2, 1);

        assert_eq!(layout.resolve(request), layout.resolve(request));
    }

    #[test]
    fn resolved_descriptors_preserve_requested_detail_tier() {
        let descriptor2 = layout2().resolve(request2(4, -2, 1)).unwrap();
        let descriptor3 = layout3().resolve(request3(-1, 2, 3, 2)).unwrap();

        assert_eq!(descriptor2.lod(), LodLevel::new(1));
        assert_eq!(descriptor3.lod(), LodLevel::new(2));
    }

    #[test]
    fn default_spatial_policy_preserves_world_coverage() {
        let layout =
            RegionLayout2::new(Extent2::try_new(32, 32).unwrap(), Vector2F64::new(2.0, 1.0))
                .unwrap();
        let lod0 = layout.resolve(request2(1, 0, 0)).unwrap();
        let lod2 = layout.resolve(request2(1, 0, 2)).unwrap();

        assert_eq!(layout.world_size().unwrap(), Vector2F64::new(64.0, 32.0));
        assert_eq!(lod0.transform().origin(), lod2.transform().origin());
        assert_eq!(lod0.cell_extent(), Extent2::try_new(32, 32).unwrap());
        assert_eq!(lod2.cell_extent(), Extent2::try_new(8, 8).unwrap());
        assert_eq!(lod2.effective_spacing(), Vector2F64::new(8.0, 4.0));
    }

    #[test]
    fn generation_region_typed_accessors_are_dimension_safe() {
        let region2 = GenerationRegion::Region2(layout2().resolve(request2(0, 0, 0)).unwrap());
        let region3 = GenerationRegion::Region3(layout3().resolve(request3(0, 0, 0, 0)).unwrap());

        assert!(region2.as_region2().is_some());
        assert!(region2.as_region3().is_none());
        assert!(region3.as_region3().is_some());
        assert!(region3.as_region2().is_none());
    }

    #[test]
    fn generation_region_lod_returns_requested_tier_for_each_dimension() {
        let region2 = GenerationRegion::Region2(layout2().resolve(request2(0, 0, 1)).unwrap());
        let region3 = GenerationRegion::Region3(layout3().resolve(request3(0, 0, 0, 2)).unwrap());

        assert_eq!(region2.lod(), LodLevel::new(1));
        assert_eq!(region3.lod(), LodLevel::new(2));
    }

    #[test]
    fn non_finite_region_origins_are_rejected() {
        let layout = RegionLayout2::new(
            Extent2::try_new(1, 1).unwrap(),
            Vector2F64::new(f64::MAX, 1.0),
        )
        .unwrap();

        assert_eq!(
            layout.resolve(request2(2, 0, 0)),
            Err(CoreError::NonFiniteRegionOrigin)
        );
    }

    fn request2(x: i64, y: i64, lod: u16) -> RegionRequest2 {
        RegionRequest2::new(RegionCoord2::new(x, y), LodLevel::new(lod))
    }

    fn request3(x: i64, y: i64, z: i64, lod: u16) -> RegionRequest3 {
        RegionRequest3::new(RegionCoord3::new(x, y, z), LodLevel::new(lod))
    }
}
