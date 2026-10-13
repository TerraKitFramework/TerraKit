//! Validated 2D and 3D grid extents.
//!
//! Extents guarantee non-zero axes and a total cell count that fits in the
//! platform address space before storage is allocated.

use crate::error::CoreError;

/// Valid dimensions for a two-dimensional grid.
///
/// An `Extent2` always has non-zero axes and a total cell count that fits in
/// `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent2 {
    width: u32,
    height: u32,
}

impl Extent2 {
    /// Creates a 2D extent from `width` and `height`.
    ///
    /// Both axes must be non-zero and their product must fit in `usize`.
    pub fn try_new(width: u32, height: u32) -> Result<Self, CoreError> {
        checked_cell_count_2(width, height)?;

        Ok(Self { width, height })
    }

    /// Creates a 2D extent from `usize` dimensions.
    ///
    /// `width` and `height` must fit in `u32`, be non-zero, and multiply to a
    /// valid platform-sized cell count.
    pub fn from_usize(width: usize, height: usize) -> Result<Self, CoreError> {
        let width = u32::try_from(width).map_err(|_| CoreError::DimensionOverflow)?;
        let height = u32::try_from(height).map_err(|_| CoreError::DimensionOverflow)?;

        Self::try_new(width, height)
    }

    /// Returns the width axis as `u32`.
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the height axis as `u32`.
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Returns the width axis as `usize`.
    pub fn width_usize(self) -> usize {
        // Invariant: validated nonzero extents prove each axis fits whenever the full cell count fits.
        usize::try_from(self.width).expect("validated u32 width must fit in usize")
    }

    /// Returns the height axis as `usize`.
    pub fn height_usize(self) -> usize {
        // Invariant: validated nonzero extents prove each axis fits whenever the full cell count fits.
        usize::try_from(self.height).expect("validated u32 height must fit in usize")
    }

    /// Returns `width * height` as `usize`.
    pub fn cell_count(self) -> usize {
        // Invariant: construction validates this exact cell-count calculation.
        checked_cell_count_2(self.width, self.height).expect("extent was validated at construction")
    }

    /// Returns the checked storage length for this extent.
    pub fn checked_len(self) -> Result<usize, CoreError> {
        Ok(self.cell_count())
    }
}

/// Valid dimensions for a three-dimensional grid.
///
/// An `Extent3` always has non-zero axes and a total cell count that fits in
/// `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent3 {
    width: u32,
    height: u32,
    depth: u32,
}

impl Extent3 {
    /// Creates a 3D extent from `width`, `height`, and `depth`.
    ///
    /// All axes must be non-zero and their product must fit in `usize`.
    pub fn try_new(width: u32, height: u32, depth: u32) -> Result<Self, CoreError> {
        checked_cell_count_3(width, height, depth)?;

        Ok(Self {
            width,
            height,
            depth,
        })
    }

    /// Creates a 3D extent from `usize` dimensions.
    ///
    /// `width`, `height`, and `depth` must fit in `u32`, be non-zero, and
    /// multiply to a valid platform-sized cell count.
    pub fn from_usize(width: usize, height: usize, depth: usize) -> Result<Self, CoreError> {
        let width = u32::try_from(width).map_err(|_| CoreError::DimensionOverflow)?;
        let height = u32::try_from(height).map_err(|_| CoreError::DimensionOverflow)?;
        let depth = u32::try_from(depth).map_err(|_| CoreError::DimensionOverflow)?;

        Self::try_new(width, height, depth)
    }

    /// Returns the width axis as `u32`.
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the height axis as `u32`.
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Returns the depth axis as `u32`.
    pub const fn depth(self) -> u32 {
        self.depth
    }

    /// Returns the width axis as `usize`.
    pub fn width_usize(self) -> usize {
        // Invariant: validated nonzero extents prove each axis fits whenever the full cell count fits.
        usize::try_from(self.width).expect("validated u32 width must fit in usize")
    }

    /// Returns the height axis as `usize`.
    pub fn height_usize(self) -> usize {
        // Invariant: validated nonzero extents prove each axis fits whenever the full cell count fits.
        usize::try_from(self.height).expect("validated u32 height must fit in usize")
    }

    /// Returns the depth axis as `usize`.
    pub fn depth_usize(self) -> usize {
        // Invariant: validated nonzero extents prove each axis fits whenever the full cell count fits.
        usize::try_from(self.depth).expect("validated u32 depth must fit in usize")
    }

    /// Returns `width * height * depth` as `usize`.
    pub fn cell_count(self) -> usize {
        // Invariant: construction validates this exact cell-count calculation.
        checked_cell_count_3(self.width, self.height, self.depth)
            .expect("extent was validated at construction")
    }

    /// Returns the checked storage length for this extent.
    pub fn checked_len(self) -> Result<usize, CoreError> {
        Ok(self.cell_count())
    }
}

fn checked_cell_count_2(width: u32, height: u32) -> Result<usize, CoreError> {
    if width == 0 || height == 0 {
        return Err(CoreError::ZeroDimension);
    }

    let length = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(CoreError::DimensionOverflow)?;

    usize::try_from(length).map_err(|_| CoreError::DimensionOverflow)
}

fn checked_cell_count_3(width: u32, height: u32, depth: u32) -> Result<usize, CoreError> {
    if width == 0 || height == 0 || depth == 0 {
        return Err(CoreError::ZeroDimension);
    }

    let length = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|value| value.checked_mul(u64::from(depth)))
        .ok_or(CoreError::DimensionOverflow)?;

    usize::try_from(length).map_err(|_| CoreError::DimensionOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extent2_try_new_accepts_valid_dimensions() {
        let extent = Extent2::try_new(4, 3).unwrap();

        assert_eq!(extent.width(), 4);
        assert_eq!(extent.height(), 3);
        assert_eq!(extent.cell_count(), 12);
    }

    #[test]
    fn extent2_try_new_rejects_zero_dimensions() {
        assert_eq!(Extent2::try_new(0, 1), Err(CoreError::ZeroDimension));
        assert_eq!(Extent2::try_new(1, 0), Err(CoreError::ZeroDimension));
    }

    #[test]
    fn extent2_from_usize_rejects_values_larger_than_u32() {
        if usize::BITS > u32::BITS {
            let width = u32::MAX as usize + 1;

            assert_eq!(
                Extent2::from_usize(width, 1),
                Err(CoreError::DimensionOverflow)
            );
        }
    }

    #[test]
    fn extent3_try_new_accepts_valid_dimensions() {
        let extent = Extent3::try_new(4, 3, 2).unwrap();

        assert_eq!(extent.width(), 4);
        assert_eq!(extent.height(), 3);
        assert_eq!(extent.depth(), 2);
        assert_eq!(extent.cell_count(), 24);
    }

    #[test]
    fn extent3_try_new_rejects_zero_dimensions() {
        assert_eq!(Extent3::try_new(1, 0, 1), Err(CoreError::ZeroDimension));
    }

    #[test]
    fn extent3_try_new_rejects_overflow() {
        assert_eq!(
            Extent3::try_new(u32::MAX, u32::MAX, u32::MAX),
            Err(CoreError::DimensionOverflow)
        );
    }
}
