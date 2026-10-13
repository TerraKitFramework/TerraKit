//! Poisson-disc object scattering stage implementation.

use terrakit_algorithms::{
    hash::hash_2d,
    scattering::{PoissonSettings, ScatterArea, ScatterError, scatter_poisson},
};
use terrakit_core::{
    HeightField, SamplingDomain, ScatterPoints, SeedDomain, TerrainResource, Vector3F64,
};
use terrakit_pipeline::{
    ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement, ResourceSet,
    StageContext, StageError, StageId, TerrainStage,
};

const STAGE_NAME: &str = "poisson_scatter";

/// Configuration of a built-in Poisson scattering stage.
#[derive(Debug, Clone, Copy)]
pub struct PoissonScatterConfig {
    /// Minimum horizontal separation of placements.
    pub radius: f32,
    /// Candidate attempts per active placement.
    pub attempts: u32,
    /// Host-controlled object prototype identifier.
    pub prototype_id: u32,
    /// Smallest uniform placement scale.
    pub min_scale: f32,
    /// Largest uniform placement scale.
    pub max_scale: f32,
    /// Whether to generate a randomized Y rotation.
    pub random_rotation: bool,
}

impl Default for PoissonScatterConfig {
    fn default() -> Self {
        Self {
            radius: 2.0,
            attempts: 30,
            prototype_id: 0,
            min_scale: 1.0,
            max_scale: 1.0,
            random_rotation: true,
        }
    }
}

/// Terrain stage that generates object placement points using Poisson-disc
/// scattering over a height field.
///
/// The stage reads an existing height field and creates a collection of
/// engine-independent scatter points. The actual object represented by each
/// point is identified by `prototype_id`.
#[derive(Debug, Clone)]
pub struct PoissonScatterStage {
    stage_id: StageId,
    input: ResourceKey,
    output: ResourceKey,

    radius: f32,
    attempts: u32,
    prototype_id: u32,
    min_scale: f32,
    max_scale: f32,
    random_rotation: bool,

    requirements: [ResourceRequirement; 1],
    products: [ResourceProduct; 1],
}

impl PoissonScatterStage {
    /// Creates a Poisson-disc scattering stage.
    pub fn new(
        stage_id: StageId,
        input: ResourceKey,
        output: ResourceKey,
        config: PoissonScatterConfig,
    ) -> Result<Self, StageError> {
        validate_configuration(
            config.radius,
            config.attempts,
            config.min_scale,
            config.max_scale,
        )?;

        if input == output {
            return Err(StageError::new(
                "Poisson scatter input and output keys must be distinct",
            ));
        }

        Ok(Self {
            stage_id,
            input,
            output,
            radius: config.radius,
            attempts: config.attempts,
            prototype_id: config.prototype_id,
            min_scale: config.min_scale,
            max_scale: config.max_scale,
            random_rotation: config.random_rotation,

            requirements: [ResourceRequirement::required(
                input,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],

            products: [ResourceProduct::create(output, ResourceKind::ScatterPoints)],
        })
    }

    /// Returns the configured input.
    pub const fn input(&self) -> ResourceKey {
        self.input
    }

    /// Returns the configured output.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns the configured radius.
    pub const fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the configured attempts.
    pub const fn attempts(&self) -> u32 {
        self.attempts
    }

    /// Returns the configured prototype id.
    pub const fn prototype_id(&self) -> u32 {
        self.prototype_id
    }

    /// Returns the configured min scale.
    pub const fn min_scale(&self) -> f32 {
        self.min_scale
    }

    /// Returns the configured max scale.
    pub const fn max_scale(&self) -> f32 {
        self.max_scale
    }

    /// Returns the configured random rotation.
    pub const fn random_rotation(&self) -> bool {
        self.random_rotation
    }
}

impl TerrainStage for PoissonScatterStage {
    fn id(&self) -> StageId {
        self.stage_id
    }

    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn requirements(&self) -> &[ResourceRequirement] {
        &self.requirements
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        context.region().as_region2().ok_or_else(|| {
            StageError::new("Poisson scatter stage requires a 2D generation region")
        })?;

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create scatter points at {:?}: resource already exists",
                self.output
            )));
        }

        let Some(height_field) = resources.height_field(self.input) else {
            return Err(StageError::new(format!(
                "Poisson scatter input {:?} is missing or is not a height field",
                self.input
            )));
        };

        let area = height_field_scatter_area(height_field)?;

        /*
         * The current terrakit-algorithms Poisson implementation works in
         * local XZ coordinates relative to the field's f64 origin.
         *
         * Deriving the seed from stage and region coordinates keeps results
         * repeatable without cloning patterns across neighboring regions.
         */
        let seed = context_seed(context, self.stage_id)?;

        let settings = PoissonSettings {
            seed,
            radius: self.radius,
            attempts: self.attempts,
            prototype_id: self.prototype_id,
            min_scale: self.min_scale,
            max_scale: self.max_scale,
            random_rotation: self.random_rotation,
        };

        let mut points = scatter_poisson(settings, area).map_err(|error| {
            scatter_stage_error("failed to generate Poisson scatter points", error)
        })?;

        // The algorithm works in XZ coordinates relative to the height field's
        // precise origin. Discard samples outside a rotated terrain footprint
        // and lift accepted placements to the interpolated surface.
        points.retain_mut(|point| {
            let Some(position) =
                sample_surface_position(height_field, point.position[0], point.position[2])
            else {
                return false;
            };
            point.position = position;
            true
        });

        let points = ScatterPoints::with_origin(height_field.transform().origin(), points);

        resources.insert(self.output, TerrainResource::ScatterPoints(points));

        Ok(())
    }
}

/// Returns a local XZ bounding box of the projected height-field footprint.
/// The bounding box is only a candidate area; positions outside the actual
/// parallelogram are rejected by `sample_surface_position`.
fn height_field_scatter_area(height_field: &HeightField) -> Result<ScatterArea, StageError> {
    if height_field.width() < 2 || height_field.height() < 2 {
        return Err(StageError::new(
            "Poisson scattering needs at least 2x2 height samples",
        ));
    }
    if height_field.sampling() != SamplingDomain::Points {
        return Err(StageError::new(
            "Poisson scattering currently requires point-sampled heights",
        ));
    }

    let transform = height_field.transform();
    let ax = transform.axis_x();
    let az = transform.axis_y();
    // An XZ projection must be invertible for a vertical-ray height sampler.
    let determinant = ax.x * az.z - ax.z * az.x;
    let axis_scale = (ax.x.abs() + ax.z.abs()) * (az.x.abs() + az.z.abs());
    if !determinant.is_finite() || determinant.abs() <= axis_scale * 1.0e-12 {
        return Err(StageError::new("height-field XZ axes cannot be inverted"));
    }

    let x_end = (height_field.width() - 1) as f64;
    let z_end = (height_field.height() - 1) as f64;
    let corners = [(0.0, 0.0), (x_end, 0.0), (0.0, z_end), (x_end, z_end)];
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_z = f64::NEG_INFINITY;
    for (x, z) in corners {
        let local_x = ax.x * x + az.x * z;
        let local_z = ax.z * x + az.z * z;
        min_x = min_x.min(local_x);
        max_x = max_x.max(local_x);
        min_z = min_z.min(local_z);
        max_z = max_z.max(local_z);
    }
    if ![min_x, max_x, min_z, max_z].iter().all(|v| v.is_finite()) {
        return Err(StageError::new(
            "height field has invalid scattering bounds",
        ));
    }
    let area = ScatterArea::new(min_x as f32, max_x as f32, min_z as f32, max_z as f32);
    area.validate()
        .map_err(|_| StageError::new("height-field scatter bounds are too large or degenerate"))?;
    Ok(area)
}

/// Interpolates an elevation and returns the surface position relative to the
/// transform origin. The query is on the base plane's XZ projection, not the
/// surface position (which may itself move horizontally along height_axis).
fn sample_surface_position(height_field: &HeightField, x: f32, z: f32) -> Option<[f32; 3]> {
    let transform = height_field.transform();
    let ax = transform.axis_x();
    let az = transform.axis_y();
    let determinant = ax.x * az.z - ax.z * az.x;
    if determinant == 0.0 || !determinant.is_finite() {
        return None;
    }

    let x = f64::from(x);
    let z = f64::from(z);
    let u = (x * az.z - z * az.x) / determinant;
    let v = (z * ax.x - x * ax.z) / determinant;
    let max_u = (height_field.width() - 1) as f64;
    let max_v = (height_field.height() - 1) as f64;
    if !u.is_finite() || !v.is_finite() || u < 0.0 || v < 0.0 || u > max_u || v > max_v {
        return None;
    }

    let x0 = u.floor() as usize;
    let z0 = v.floor() as usize;
    let x1 = (x0 + 1).min(height_field.width() - 1);
    let z1 = (z0 + 1).min(height_field.height() - 1);
    let tx = u - x0 as f64;
    let tz = v - z0 as f64;
    let h00 = f64::from(height_field.get(x0, z0)?);
    let h10 = f64::from(height_field.get(x1, z0)?);
    let h01 = f64::from(height_field.get(x0, z1)?);
    let h11 = f64::from(height_field.get(x1, z1)?);
    let h = (h00 * (1.0 - tx) + h10 * tx) * (1.0 - tz) + (h01 * (1.0 - tx) + h11 * tx) * tz;
    let height_axis = height_field.height_axis();
    let local = Vector3F64::new(
        ax.x * u + az.x * v + height_axis.x * h,
        ax.y * u + az.y * v + height_axis.y * h,
        ax.z * u + az.z * v + height_axis.z * h,
    );
    let local = [local.x as f32, local.y as f32, local.z as f32];
    if local.iter().all(|v| v.is_finite()) {
        Some(local)
    } else {
        None
    }
}

/// Derives a stable, independent Poisson seed for each stage and region.
/// Region coordinates are hashed explicitly to avoid cloned local patterns
/// across adjacent chunks while retaining deterministic regeneration.
fn context_seed(context: &StageContext, stage_id: StageId) -> Result<u64, StageError> {
    let coordinate = context.region2()?.coordinate();
    let stage_seed = context.derive_seed(SeedDomain::new(stage_id.0));
    Ok(hash_2d(stage_seed, coordinate.x, coordinate.y))
}

fn validate_configuration(
    radius: f32,
    attempts: u32,
    min_scale: f32,
    max_scale: f32,
) -> Result<(), StageError> {
    if !radius.is_finite() || radius <= 0.0 {
        return Err(StageError::new(
            "Poisson scatter radius must be finite and greater than zero",
        ));
    }

    if !(1..=10_000).contains(&attempts) {
        return Err(StageError::new(
            "Poisson scatter attempts must be within 1..=10000",
        ));
    }

    if !min_scale.is_finite() || min_scale < 0.0 {
        return Err(StageError::new(
            "Poisson scatter minimum scale must be finite and non-negative",
        ));
    }

    if !max_scale.is_finite() {
        return Err(StageError::new(
            "Poisson scatter maximum scale must be finite",
        ));
    }

    if max_scale < min_scale {
        return Err(StageError::new(
            "Poisson scatter maximum scale must be greater than or equal to minimum scale",
        ));
    }

    Ok(())
}

fn scatter_stage_error(operation: &str, error: ScatterError) -> StageError {
    StageError::new(format!("{operation}: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use terrakit_core::{
        Extent2, GenerationRegion, GenerationSeed, LodLevel, RegionCoord2, RegionLayout2,
        RegionRequest2, Vector2F64,
    };

    fn context2(width: u32, height: u32) -> StageContext {
        let layout = RegionLayout2::new(
            Extent2::try_new(width, height).unwrap(),
            Vector2F64::new(1.0, 1.0),
        )
        .unwrap();

        let region = GenerationRegion::Region2(
            layout
                .resolve(RegionRequest2::new(RegionCoord2::ZERO, LodLevel::HIGHEST))
                .unwrap(),
        );

        StageContext::new(GenerationSeed::new(123), region)
    }

    #[test]
    fn bilinear_sampler_handles_precise_origin_and_rotation() {
        use terrakit_core::{GridTransform2, SamplingDomain};
        let origin = Vector3F64::new(1_000_000_000.0, 50.0, 1_000_000_000.0);
        let transform =
            GridTransform2::from_axes(origin, Vector3F64::Z, Vector3F64::new(-1.0, 0.0, 0.0))
                .unwrap();
        let field = HeightField::from_values(
            Extent2::try_new(2, 2).unwrap(),
            vec![1.0, 3.0, 5.0, 7.0],
            transform,
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .unwrap();
        assert_eq!(
            height_field_scatter_area(&field).unwrap(),
            ScatterArea::new(-1.0, 0.0, 0.0, 1.0)
        );
        let placement = sample_surface_position(&field, -0.5, 0.25).unwrap();
        assert!((placement[0] + 0.5).abs() < 1e-6);
        assert!((placement[1] - 3.5).abs() < 1e-6);
        assert!((placement[2] - 0.25).abs() < 1e-6);
        assert!(sample_surface_position(&field, 0.5, 0.25).is_none());
    }

    #[test]
    fn generates_scatter_points() {
        let context = context2(20, 20);

        let mut height_stage =
            crate::FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, 0.0).unwrap();

        let mut scatter_stage = PoissonScatterStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            ResourceKey::SCATTER_POINTS,
            PoissonScatterConfig {
                radius: 2.0,
                attempts: 30,
                prototype_id: 42,
                ..PoissonScatterConfig::default()
            },
        )
        .unwrap();

        let mut resources = ResourceSet::new();

        // First create the terrain.
        height_stage.execute(&context, &mut resources).unwrap();

        // Then scatter objects across the terrain.
        scatter_stage.execute(&context, &mut resources).unwrap();

        let Some(TerrainResource::ScatterPoints(points)) =
            resources.get(ResourceKey::SCATTER_POINTS)
        else {
            panic!("expected scatter points resource");
        };

        println!("Generated {} scatter points", points.len());

        assert!(!points.is_empty());
    }
}
