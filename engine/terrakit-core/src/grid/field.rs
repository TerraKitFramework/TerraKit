//! Spatial grid fields.
//!
//! A field combines dense grid storage with a grid-to-world transform and a
//! sampling-domain convention, making raw samples meaningful in world space.

use super::{Grid2, Grid3, GridTransform2, GridTransform3};
use crate::{error::CoreError, grid::Extent2, grid::Extent3, math::Vector3F64};

/// Describes where grid samples live relative to the grid lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingDomain {
    /// Each value is located directly at an integer grid coordinate.
    Points,

    /// Each value describes the cell area or volume beginning at a grid coordinate.
    Cells,
}

/// A two-dimensional grid with spatial meaning.
///
/// `Field2<T>` is the reusable foundation for height fields, masks, biome
/// maps, and other 2D terrain layers. It owns storage, a world-space transform,
/// and the sampling convention for the values.
#[derive(Debug, Clone, PartialEq)]
pub struct Field2<T> {
    grid: Grid2<T>,
    transform: GridTransform2,
    sampling: SamplingDomain,
}

impl<T> Field2<T> {
    /// Creates a 2D field from `grid`, `transform`, and `sampling` metadata.
    pub fn new(grid: Grid2<T>, transform: GridTransform2, sampling: SamplingDomain) -> Self {
        Self {
            grid,
            transform,
            sampling,
        }
    }

    /// Creates a 2D field from `extent`, backing `values`, `transform`, and `sampling`.
    ///
    /// The number of `values` must exactly match `extent.cell_count()`.
    pub fn from_vec(
        extent: Extent2,
        values: Vec<T>,
        transform: GridTransform2,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        let grid = Grid2::from_vec(extent, values)?;

        Ok(Self::new(grid, transform, sampling))
    }

    /// Returns the validated 2D extent of this field.
    pub fn extent(&self) -> Extent2 {
        self.grid.extent()
    }

    /// Returns the field width as `usize`.
    pub fn width(&self) -> usize {
        self.grid.width()
    }

    /// Returns the field height as `usize`.
    pub fn height(&self) -> usize {
        self.grid.height()
    }

    /// Returns the number of samples in the field.
    pub fn len(&self) -> usize {
        self.grid.len()
    }

    /// Returns true when the field stores no samples.
    ///
    /// Valid extents are non-empty, so this is normally false for constructed
    /// fields.
    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }

    /// Returns the world-space transform applied to grid coordinates.
    pub fn transform(&self) -> GridTransform2 {
        self.transform
    }

    /// Returns whether values are point samples or cell samples.
    pub fn sampling(&self) -> SamplingDomain {
        self.sampling
    }

    /// Returns the underlying dense 2D grid.
    pub fn grid(&self) -> &Grid2<T> {
        &self.grid
    }

    /// Returns all values in X-contiguous row order.
    pub fn values(&self) -> &[T] {
        self.grid.values()
    }

    /// Returns mutable access to all values in X-contiguous row order.
    pub fn values_mut(&mut self) -> &mut [T] {
        self.grid.values_mut()
    }

    /// Returns the value at `x`, `y`, or `None` when out of bounds.
    pub fn get(&self, x: u32, y: u32) -> Option<&T> {
        self.grid.get(x, y)
    }

    /// Returns the value at `usize` coordinate `x`, `y`, or `None` when out of bounds.
    pub fn get_usize(&self, x: usize, y: usize) -> Option<&T> {
        self.grid.get_usize(x, y)
    }

    /// Writes `value` at `x`, `y`, returning a bounds error if the coordinate is invalid.
    pub fn try_set(&mut self, x: usize, y: usize, value: T) -> Result<(), CoreError> {
        let width = self.width();
        let height = self.height();

        let Some(cell) = self.grid.get_usize_mut(x, y) else {
            return Err(CoreError::CoordinateOutOfBounds2 {
                x,
                y,
                width,
                height,
            });
        };

        *cell = value;
        Ok(())
    }

    /// Returns the world-space position for the sample at `x`, `y`.
    ///
    /// Returns `None` when the coordinate is outside the field extent.
    pub fn sample_position(&self, x: usize, y: usize) -> Option<Vector3F64> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;

        self.grid.index_of(x, y)?;

        Some(self.transform.position_at(f64::from(x), f64::from(y)))
    }

    /// Consumes the field and returns its underlying grid storage.
    pub fn into_grid(self) -> Grid2<T> {
        self.grid
    }
}

impl<T: Clone> Field2<T> {
    /// Creates a 2D field of `extent` with every value cloned from `value`.
    ///
    /// `transform` places grid coordinates in world space and `sampling`
    /// declares whether values live on points or cells.
    pub fn filled(
        extent: Extent2,
        value: T,
        transform: GridTransform2,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        let grid = Grid2::filled(extent, value)?;

        Ok(Self::new(grid, transform, sampling))
    }
}

/// A three-dimensional grid with spatial meaning.
///
/// `Field3<T>` is the reusable foundation for density fields, voxel volumes,
/// material volumes, and other 3D terrain layers.
#[derive(Debug, Clone, PartialEq)]
pub struct Field3<T> {
    grid: Grid3<T>,
    transform: GridTransform3,
    sampling: SamplingDomain,
}

impl<T> Field3<T> {
    /// Creates a 3D field from `grid`, `transform`, and `sampling` metadata.
    pub fn new(grid: Grid3<T>, transform: GridTransform3, sampling: SamplingDomain) -> Self {
        Self {
            grid,
            transform,
            sampling,
        }
    }

    /// Creates a 3D field from `extent`, backing `values`, `transform`, and `sampling`.
    ///
    /// The number of `values` must exactly match `extent.cell_count()`.
    pub fn from_vec(
        extent: Extent3,
        values: Vec<T>,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        let grid = Grid3::from_vec(extent, values)?;

        Ok(Self::new(grid, transform, sampling))
    }

    /// Returns the validated 3D extent of this field.
    pub fn extent(&self) -> Extent3 {
        self.grid.extent()
    }

    /// Returns the field width as `usize`.
    pub fn width(&self) -> usize {
        self.grid.width()
    }

    /// Returns the field height as `usize`.
    pub fn height(&self) -> usize {
        self.grid.height()
    }

    /// Returns the field depth as `usize`.
    pub fn depth(&self) -> usize {
        self.grid.depth()
    }

    /// Returns the number of samples in the field.
    pub fn len(&self) -> usize {
        self.grid.len()
    }

    /// Returns true when the field stores no samples.
    ///
    /// Valid extents are non-empty, so this is normally false for constructed
    /// fields.
    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }

    /// Returns the world-space transform applied to grid coordinates.
    pub fn transform(&self) -> GridTransform3 {
        self.transform
    }

    /// Returns whether values are point samples or cell samples.
    pub fn sampling(&self) -> SamplingDomain {
        self.sampling
    }

    /// Returns the underlying dense 3D grid.
    pub fn grid(&self) -> &Grid3<T> {
        &self.grid
    }

    /// Returns all values in X-contiguous layer order.
    pub fn values(&self) -> &[T] {
        self.grid.values()
    }

    /// Returns mutable access to all values in X-contiguous layer order.
    pub fn values_mut(&mut self) -> &mut [T] {
        self.grid.values_mut()
    }

    /// Returns the value at `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<&T> {
        self.grid.get(x, y, z)
    }

    /// Returns the value at `usize` coordinate `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get_usize(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        self.grid.get_usize(x, y, z)
    }

    /// Writes `value` at `x`, `y`, `z`, returning a bounds error if the coordinate is invalid.
    pub fn try_set(&mut self, x: usize, y: usize, z: usize, value: T) -> Result<(), CoreError> {
        let width = self.width();
        let height = self.height();
        let depth = self.depth();

        let Some(cell) = self.grid.get_usize_mut(x, y, z) else {
            return Err(CoreError::CoordinateOutOfBounds3 {
                x,
                y,
                z,
                width,
                height,
                depth,
            });
        };

        *cell = value;
        Ok(())
    }

    /// Returns the world-space position for the sample at `x`, `y`, `z`.
    ///
    /// Returns `None` when the coordinate is outside the field extent.
    pub fn sample_position(&self, x: usize, y: usize, z: usize) -> Option<Vector3F64> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;
        let z = u32::try_from(z).ok()?;

        self.grid.index_of(x, y, z)?;

        Some(
            self.transform
                .position_at(f64::from(x), f64::from(y), f64::from(z)),
        )
    }

    /// Consumes the field and returns its underlying grid storage.
    pub fn into_grid(self) -> Grid3<T> {
        self.grid
    }
}

impl<T: Clone> Field3<T> {
    /// Creates a 3D field of `extent` with every value cloned from `value`.
    ///
    /// `transform` places grid coordinates in world space and `sampling`
    /// declares whether values live on points or cells.
    pub fn filled(
        extent: Extent3,
        value: T,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        let grid = Grid3::filled(extent, value)?;

        Ok(Self::new(grid, transform, sampling))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field2_carries_grid_transform_and_sampling_domain() {
        let extent = Extent2::try_new(2, 2).unwrap();
        let transform = GridTransform2::from_spacing(3.0).unwrap();
        let field = Field2::filled(extent, 1, transform, SamplingDomain::Points).unwrap();

        assert_eq!(field.values(), &[1, 1, 1, 1]);
        assert_eq!(field.transform(), transform);
        assert_eq!(field.sampling(), SamplingDomain::Points);
        assert_eq!(
            field.sample_position(1, 1),
            Some(Vector3F64::new(3.0, 0.0, 3.0))
        );
    }

    #[test]
    fn field2_try_set_reports_bounds_errors() {
        let extent = Extent2::try_new(2, 2).unwrap();
        let mut field = Field2::filled(
            extent,
            1,
            GridTransform2::identity_xz(),
            SamplingDomain::Cells,
        )
        .unwrap();

        assert_eq!(
            field.try_set(2, 0, 4),
            Err(CoreError::CoordinateOutOfBounds2 {
                x: 2,
                y: 0,
                width: 2,
                height: 2,
            })
        );
    }

    #[test]
    fn field3_carries_grid_transform_and_sampling_domain() {
        let extent = Extent3::try_new(2, 2, 2).unwrap();
        let transform = GridTransform3::from_spacing(2.0).unwrap();
        let field = Field3::filled(extent, 5, transform, SamplingDomain::Cells).unwrap();

        assert_eq!(field.len(), 8);
        assert_eq!(field.sampling(), SamplingDomain::Cells);
        assert_eq!(
            field.sample_position(1, 1, 1),
            Some(Vector3F64::new(2.0, 2.0, 2.0))
        );
    }

    #[test]
    fn field3_try_set_reports_bounds_errors() {
        let extent = Extent3::try_new(2, 2, 2).unwrap();
        let mut field = Field3::filled(
            extent,
            1,
            GridTransform3::identity(),
            SamplingDomain::Points,
        )
        .unwrap();

        assert_eq!(
            field.try_set(0, 2, 0, 4),
            Err(CoreError::CoordinateOutOfBounds3 {
                x: 0,
                y: 2,
                z: 0,
                width: 2,
                height: 2,
                depth: 2,
            })
        );
    }
}
