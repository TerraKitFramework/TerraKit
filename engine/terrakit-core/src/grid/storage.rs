//! Dense X-contiguous grid storage.
//!
//! `Grid2` and `Grid3` own validated buffers and provide coordinate-based
//! indexing without spatial metadata.

use super::{Extent2, Extent3};
use crate::error::CoreError;

/// A two-dimensional X-contiguous grid.
///
/// Memory layout:
///
/// `index = y * width + x`
#[derive(Debug, Clone, PartialEq)]
pub struct Grid2<T> {
    extent: Extent2,
    values: Vec<T>,
}

impl<T> Grid2<T> {
    /// Creates a grid from exactly `extent.cell_count()` values.
    pub fn from_vec(extent: Extent2, values: Vec<T>) -> Result<Self, CoreError> {
        let expected = extent.cell_count();

        if values.len() != expected {
            return Err(CoreError::InvalidBufferLength {
                expected,
                actual: values.len(),
            });
        }

        Ok(Self { extent, values })
    }

    /// Returns the validated 2D extent used by this grid.
    pub fn extent(&self) -> Extent2 {
        self.extent
    }

    /// Returns the grid width as `usize`.
    pub fn width(&self) -> usize {
        self.extent.width_usize()
    }

    /// Returns the grid height as `usize`.
    pub fn height(&self) -> usize {
        self.extent.height_usize()
    }

    /// Returns the number of stored values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true when the grid stores no values.
    ///
    /// Valid extents are non-empty, so this is normally false for constructed
    /// grids.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns all values in X-contiguous row order.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// Returns mutable access to all values in X-contiguous row order.
    pub fn values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }

    /// Returns the value at `x`, `y`, or `None` when out of bounds.
    pub fn get(&self, x: u32, y: u32) -> Option<&T> {
        let index = self.index_of(x, y)?;
        self.values.get(index)
    }

    /// Returns the value at `x`, `y` supplied as `usize`, or `None` when out of bounds.
    pub fn get_usize(&self, x: usize, y: usize) -> Option<&T> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;

        self.get(x, y)
    }

    /// Returns mutable access to the value at `x`, `y`, or `None` when out of bounds.
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut T> {
        let index = self.index_of(x, y)?;
        self.values.get_mut(index)
    }

    /// Returns mutable access to the `usize` coordinate `x`, `y`, or `None` when out of bounds.
    pub fn get_usize_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;

        self.get_mut(x, y)
    }

    /// Returns the row-major storage index for `x`, `y`.
    pub fn index_of(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.extent.width() || y >= self.extent.height() {
            return None;
        }

        let index = u64::from(y) * u64::from(self.extent.width()) + u64::from(x);

        usize::try_from(index).ok()
    }

    /// Consumes the grid and returns its backing values.
    pub fn into_vec(self) -> Vec<T> {
        self.values
    }
}

impl<T: Clone> Grid2<T> {
    /// Creates a grid of `extent` where every cell is cloned from `value`.
    pub fn filled(extent: Extent2, value: T) -> Result<Self, CoreError> {
        let length = extent.cell_count();

        Ok(Self {
            extent,
            values: vec![value; length],
        })
    }
}

/// A three-dimensional X-contiguous grid.
///
/// Memory layout:
///
/// `index = (z * height + y) * width + x`
#[derive(Debug, Clone, PartialEq)]
pub struct Grid3<T> {
    extent: Extent3,
    values: Vec<T>,
}

impl<T> Grid3<T> {
    /// Creates a grid from exactly `extent.cell_count()` values.
    pub fn from_vec(extent: Extent3, values: Vec<T>) -> Result<Self, CoreError> {
        let expected = extent.cell_count();

        if values.len() != expected {
            return Err(CoreError::InvalidBufferLength {
                expected,
                actual: values.len(),
            });
        }

        Ok(Self { extent, values })
    }

    /// Returns the validated 3D extent used by this grid.
    pub fn extent(&self) -> Extent3 {
        self.extent
    }

    /// Returns the grid width as `usize`.
    pub fn width(&self) -> usize {
        self.extent.width_usize()
    }

    /// Returns the grid height as `usize`.
    pub fn height(&self) -> usize {
        self.extent.height_usize()
    }

    /// Returns the grid depth as `usize`.
    pub fn depth(&self) -> usize {
        self.extent.depth_usize()
    }

    /// Returns the number of stored values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true when the grid stores no values.
    ///
    /// Valid extents are non-empty, so this is normally false for constructed
    /// grids.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns all values in X-contiguous layer order.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// Returns mutable access to all values in X-contiguous layer order.
    pub fn values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }

    /// Returns the value at `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<&T> {
        let index = self.index_of(x, y, z)?;
        self.values.get(index)
    }

    /// Returns the value at `usize` coordinate `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get_usize(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;
        let z = u32::try_from(z).ok()?;

        self.get(x, y, z)
    }

    /// Returns mutable access to `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get_mut(&mut self, x: u32, y: u32, z: u32) -> Option<&mut T> {
        let index = self.index_of(x, y, z)?;
        self.values.get_mut(index)
    }

    /// Returns mutable access to the `usize` coordinate `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get_usize_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut T> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;
        let z = u32::try_from(z).ok()?;

        self.get_mut(x, y, z)
    }

    /// Returns the X-contiguous storage index for `x`, `y`, `z`.
    pub fn index_of(&self, x: u32, y: u32, z: u32) -> Option<usize> {
        if x >= self.extent.width() || y >= self.extent.height() || z >= self.extent.depth() {
            return None;
        }

        let index = (u64::from(z) * u64::from(self.extent.height()) + u64::from(y))
            * u64::from(self.extent.width())
            + u64::from(x);

        usize::try_from(index).ok()
    }

    /// Consumes the grid and returns its backing values.
    pub fn into_vec(self) -> Vec<T> {
        self.values
    }
}

impl<T: Clone> Grid3<T> {
    /// Creates a grid of `extent` where every cell is cloned from `value`.
    pub fn filled(extent: Extent3, value: T) -> Result<Self, CoreError> {
        let length = extent.cell_count();

        Ok(Self {
            extent,
            values: vec![value; length],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid2_from_vec_rejects_invalid_buffer_lengths() {
        let extent = Extent2::try_new(2, 2).unwrap();
        let error = Grid2::from_vec(extent, vec![1, 2, 3]).unwrap_err();

        assert_eq!(
            error,
            CoreError::InvalidBufferLength {
                expected: 4,
                actual: 3,
            }
        );
    }

    #[test]
    fn grid2_indexes_x_contiguous_rows() {
        let extent = Extent2::try_new(3, 2).unwrap();
        let grid = Grid2::from_vec(extent, vec![0, 1, 2, 3, 4, 5]).unwrap();

        assert_eq!(grid.index_of(2, 1), Some(5));
        assert_eq!(grid.get(2, 1), Some(&5));
        assert_eq!(grid.len(), 6);
    }

    #[test]
    fn grid2_mutates_cells_by_coordinate() {
        let extent = Extent2::try_new(2, 2).unwrap();
        let mut grid = Grid2::filled(extent, 0).unwrap();

        *grid.get_mut(1, 1).unwrap() = 7;

        assert_eq!(grid.values(), &[0, 0, 0, 7]);
    }

    #[test]
    fn grid2_returns_none_for_out_of_bounds_coordinates() {
        let extent = Extent2::try_new(2, 2).unwrap();
        let grid = Grid2::filled(extent, 0).unwrap();

        assert_eq!(grid.index_of(2, 0), None);
        assert_eq!(grid.get_usize(0, 3), None);
    }

    #[test]
    fn grid3_indexes_x_contiguous_layers() {
        let values = (0..24).collect();
        let extent = Extent3::try_new(3, 4, 2).unwrap();
        let grid = Grid3::from_vec(extent, values).unwrap();

        assert_eq!(grid.index_of(1, 2, 1), Some(19));
        assert_eq!(grid.get(1, 2, 1), Some(&19));
        assert_eq!(grid.len(), 24);
    }

    #[test]
    fn grid3_mutates_cells_by_coordinate() {
        let extent = Extent3::try_new(2, 2, 2).unwrap();
        let mut grid = Grid3::filled(extent, 0).unwrap();

        *grid.get_usize_mut(1, 1, 1).unwrap() = 9;

        assert_eq!(grid.into_vec(), vec![0, 0, 0, 0, 0, 0, 0, 9]);
    }
}
