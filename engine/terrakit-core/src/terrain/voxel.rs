//! Voxel-volume terrain resource.
//!
//! A voxel volume wraps a 3D spatial field of compact voxel identifiers.

use crate::{
    error::CoreError,
    grid::{Extent3, Field3, GridTransform3, SamplingDomain},
};

/// Identifier for a voxel material or occupancy class.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct VoxelId(
    /// Raw voxel material or occupancy value.
    pub u32,
);

impl VoxelId {
    /// Conventional empty voxel identifier.
    pub const EMPTY: Self = Self(0);

    /// Creates a voxel identifier from raw `value`.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw voxel identifier value.
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A dense 3D volume of voxel identifiers.
///
/// Values are stored as contiguous `VoxelId` samples in X-contiguous layer
/// order: `index = (z * height + y) * width + x`. ABI callers may represent
/// each `VoxelId` as its raw `u32` value.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelVolume {
    field: Field3<VoxelId>,
}

impl VoxelVolume {
    /// Creates an empty-filled voxel volume of `extent`.
    ///
    /// `transform` places grid coordinates in world space and `sampling`
    /// defines whether values live on points or cells.
    pub fn new(
        extent: Extent3,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        Self::filled(extent, VoxelId::EMPTY, transform, sampling)
    }

    /// Creates a cell-sampled empty voxel volume from `width`, `height`, and `depth`.
    pub fn from_dimensions(width: usize, height: usize, depth: usize) -> Result<Self, CoreError> {
        let extent = Extent3::from_usize(width, height, depth)?;

        Self::new(extent, GridTransform3::identity(), SamplingDomain::Cells)
    }

    /// Creates a voxel volume of `extent` filled with `value`.
    ///
    /// `transform` places samples in world space and `sampling` declares the
    /// point or cell convention.
    pub fn filled(
        extent: Extent3,
        value: VoxelId,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        let field = Field3::filled(extent, value, transform, sampling)?;

        Ok(Self { field })
    }

    /// Creates a voxel volume from explicit `values`.
    ///
    /// `values` must match `extent.cell_count()`, `transform` places samples in
    /// world space, and `sampling` declares the point or cell convention.
    pub fn from_values(
        extent: Extent3,
        values: Vec<VoxelId>,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        let field = Field3::from_vec(extent, values, transform, sampling)?;

        Ok(Self { field })
    }

    /// Wraps an existing spatial `field` of voxel identifiers.
    pub fn from_field(field: Field3<VoxelId>) -> Self {
        Self { field }
    }

    /// Returns the validated 3D extent of the voxel samples.
    pub fn extent(&self) -> Extent3 {
        self.field.extent()
    }

    /// Returns the voxel volume width as `usize`.
    pub fn width(&self) -> usize {
        self.field.width()
    }

    /// Returns the voxel volume height as `usize`.
    pub fn height(&self) -> usize {
        self.field.height()
    }

    /// Returns the voxel volume depth as `usize`.
    pub fn depth(&self) -> usize {
        self.field.depth()
    }

    /// Returns the number of voxel samples.
    pub fn len(&self) -> usize {
        self.field.len()
    }

    /// Returns true when the voxel volume stores no samples.
    ///
    /// Valid extents are non-empty, so this is normally false for constructed
    /// voxel volumes.
    pub fn is_empty(&self) -> bool {
        self.field.is_empty()
    }

    /// Returns all voxel identifiers in X-contiguous layer order.
    ///
    /// The storage index is `(z * height + y) * width + x`. When accessed
    /// through a future foreign-function view, this slice is borrowed from the
    /// owning generation result.
    pub fn values(&self) -> &[VoxelId] {
        self.field.values()
    }

    /// Returns the underlying typed spatial field.
    pub fn field(&self) -> &Field3<VoxelId> {
        &self.field
    }

    /// Returns the voxel at `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<VoxelId> {
        self.field.get_usize(x, y, z).copied()
    }

    /// Writes `value` at `x`, `y`, `z`, returning a bounds error when invalid.
    pub fn try_set(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        value: VoxelId,
    ) -> Result<(), CoreError> {
        self.field.try_set(x, y, z, value)
    }

    /// Consumes the voxel volume and returns its underlying spatial field.
    pub fn into_field(self) -> Field3<VoxelId> {
        self.field
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voxel_id_exposes_empty_and_raw_value() {
        assert_eq!(VoxelId::EMPTY.value(), 0);
        assert_eq!(VoxelId::new(42).value(), 42);
    }

    #[test]
    fn voxel_volume_defaults_to_empty_voxels() {
        let volume = VoxelVolume::from_dimensions(2, 2, 2).unwrap();

        assert_eq!(volume.values(), &[VoxelId::EMPTY; 8]);
        assert_eq!(volume.field().sampling(), SamplingDomain::Cells);
    }

    #[test]
    fn voxel_volume_try_set_reports_bounds_errors() {
        let mut volume = VoxelVolume::from_dimensions(2, 2, 2).unwrap();

        assert_eq!(
            volume.try_set(2, 0, 0, VoxelId::new(1)),
            Err(CoreError::CoordinateOutOfBounds3 {
                x: 2,
                y: 0,
                z: 0,
                width: 2,
                height: 2,
                depth: 2,
            })
        );
    }
}
