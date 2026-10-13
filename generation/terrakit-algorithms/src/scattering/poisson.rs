// Deterministic Poisson-disc object scattering.
// Poisson-disc sampling prevents objects from appearing too close together.

use super::random::{ScatterRng, random_y_rotation};

use terrakit_core::ScatterPoint;

use super::types::{ScatterArea, ScatterError};

/// Configuration for Poisson-disc scattering.
#[derive(Debug, Clone, Copy)]
pub struct PoissonSettings {
    /// Random seed.
    pub seed: u64,
    /// Minimum horizontal distance between objects.
    pub radius: f32,
    /// Candidate attempts for each active point.
    pub attempts: u32,
    /// Object prototype identifier.
    pub prototype_id: u32,
    /// Minimum uniform scale.
    pub min_scale: f32,
    /// Maximum uniform scale.
    pub max_scale: f32,
    /// Randomize yaw.
    pub random_rotation: bool,
}

impl Default for PoissonSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 5.0,
            attempts: 30,
            prototype_id: 0,
            min_scale: 1.0,
            max_scale: 1.0,
            random_rotation: true,
        }
    }
}

impl PoissonSettings {
    /// Validates the settings.
    pub fn validate(&self) -> Result<(), ScatterError> {
        if !self.radius.is_finite() || self.radius <= 0.0 {
            return Err(ScatterError::InvalidRadius);
        }

        if !(1..=10_000).contains(&self.attempts) {
            return Err(ScatterError::InvalidAttempts);
        }

        if !self.min_scale.is_finite()
            || !self.max_scale.is_finite()
            || self.min_scale < 0.0
            || self.max_scale < self.min_scale
        {
            return Err(ScatterError::InvalidSettings("invalid scale range"));
        }

        Ok(())
    }
}

/// Generates Poisson-disc distributed points in a local XZ rectangle.
///
/// Each generated placement is initially at local Y=0, so terrain stages
/// must lift it to the actual sampled surface before publishing resources.
pub fn scatter_poisson(
    settings: PoissonSettings,
    area: ScatterArea,
) -> Result<Vec<ScatterPoint>, ScatterError> {
    settings.validate()?;
    area.validate()?;

    let mut rng = ScatterRng::new(settings.seed);

    let cell_size = settings.radius / 2.0_f32.sqrt();
    if !cell_size.is_finite() || cell_size <= 0.0 {
        return Err(ScatterError::InvalidRadius);
    }

    // A densely sampled large region could otherwise exhaust host memory.
    const MAX_GRID_CELLS: usize = 4_194_304;
    let grid_dimension = |span: f32| -> Result<usize, ScatterError> {
        let cells = f64::from(span) / f64::from(cell_size);
        if !cells.is_finite() || cells.ceil() >= MAX_GRID_CELLS as f64 {
            return Err(ScatterError::GridTooLarge);
        }
        Ok(cells.ceil() as usize + 1)
    };
    let grid_width = grid_dimension(area.width())?;
    let grid_height = grid_dimension(area.depth())?;
    let total = grid_width
        .checked_mul(grid_height)
        .filter(|&count| count <= MAX_GRID_CELLS)
        .ok_or(ScatterError::GridTooLarge)?;

    let mut grid: Vec<Option<usize>> = Vec::new();
    grid.try_reserve_exact(total)
        .map_err(|_| ScatterError::GridTooLarge)?;
    grid.resize(total, None);

    let mut points: Vec<[f32; 2]> = Vec::new();
    let mut active: Vec<usize> = Vec::new();
    const MAX_TOTAL_CANDIDATES: u64 = 20_000_000;
    let mut candidate_count = 0u64;

    // Initial point.
    let initial_x = rng.range_f32(area.min_x, area.max_x);

    let initial_z = rng.range_f32(area.min_z, area.max_z);

    let initial = [initial_x, initial_z];

    points.push(initial);
    active.push(0);

    let gx = grid_x(initial[0], &area, cell_size);

    let gz = grid_z(initial[1], &area, cell_size);

    grid[gz * grid_width + gx] = Some(0);

    while !active.is_empty() {
        let active_position = (rng.next_u64() % active.len() as u64) as usize;

        let point_index = active[active_position];

        let origin = points[point_index];

        let mut found = false;

        for _ in 0..settings.attempts {
            candidate_count += 1;
            if candidate_count > MAX_TOTAL_CANDIDATES {
                return Err(ScatterError::BudgetExceeded);
            }
            let angle = rng.next_f32() * std::f32::consts::TAU;

            let distance = settings.radius * (1.0 + rng.next_f32());

            let candidate = [
                origin[0] + angle.cos() * distance,
                origin[1] + angle.sin() * distance,
            ];

            if !inside_area(candidate, &area) {
                continue;
            }

            if !is_valid_candidate(
                candidate,
                &points,
                &grid,
                grid_width,
                cell_size,
                &area,
                settings.radius,
            ) {
                continue;
            }

            let new_index = points.len();

            points.push(candidate);
            active.push(new_index);

            let gx = grid_x(candidate[0], &area, cell_size);

            let gz = grid_z(candidate[1], &area, cell_size);

            grid[gz * grid_width + gx] = Some(new_index);

            found = true;

            break;
        }

        if !found {
            active.swap_remove(active_position);
        }
    }

    let mut result = Vec::with_capacity(points.len());

    for point in points {
        let scale = rng.range_f32(settings.min_scale, settings.max_scale);

        let rotation = if settings.random_rotation {
            random_y_rotation(&mut rng)
        } else {
            [0.0, 0.0, 0.0, 1.0]
        };

        result.push(ScatterPoint {
            position: [point[0], 0.0, point[1]],
            rotation,
            scale: [scale, scale, scale],
            prototype_id: settings.prototype_id,
        });
    }

    Ok(result)
}

// Converts world X into a grid coordinate.
fn grid_x(x: f32, area: &ScatterArea, cell_size: f32) -> usize {
    ((x - area.min_x) / cell_size).floor() as usize
}

// Converts world Z into a grid coordinate.
fn grid_z(z: f32, area: &ScatterArea, cell_size: f32) -> usize {
    ((z - area.min_z) / cell_size).floor() as usize
}

// Checks whether a point is inside the scattering area.
fn inside_area(point: [f32; 2], area: &ScatterArea) -> bool {
    point[0] >= area.min_x
        && point[0] < area.max_x
        && point[1] >= area.min_z
        && point[1] < area.max_z
}

// Checks whether a candidate is sufficiently far from all nearby points.
fn is_valid_candidate(
    candidate: [f32; 2],
    points: &[[f32; 2]],
    grid: &[Option<usize>],
    grid_width: usize,
    cell_size: f32,
    area: &ScatterArea,
    radius: f32,
) -> bool {
    let gx = grid_x(candidate[0], area, cell_size);

    let gz = grid_z(candidate[1], area, cell_size);

    let radius_squared = radius * radius;

    let min_x = gx.saturating_sub(2);

    let max_x = gx.saturating_add(2).min(grid_width - 1);

    let grid_height = grid.len() / grid_width;

    let min_z = gz.saturating_sub(2);

    let max_z = gz.saturating_add(2).min(grid_height - 1);

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            let index = z * grid_width + x;

            if let Some(point_index) = grid[index] {
                let point = points[point_index];

                let dx = candidate[0] - point[0];

                let dz = candidate[1] - point[1];

                let distance_squared = dx * dx + dz * dz;

                if distance_squared < radius_squared {
                    return false;
                }
            }
        }
    }

    true
}
