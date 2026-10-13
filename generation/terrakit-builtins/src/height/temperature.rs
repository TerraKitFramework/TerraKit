//! Temperature map stage implementation.

use terrakit_algorithms::noise::{Noise2D, NoiseAlgorithm};
use terrakit_core::{HeightField, SamplingDomain, SeedDomain, TerrainResource, Vector3F64};
use terrakit_pipeline::{
    ResourceKey, ResourceKind, ResourceProduct, ResourceSet, StageContext, StageError, StageId,
    TerrainStage,
};

const STAGE_NAME: &str = "biome_temperature";

/// Terrain stage that generates a temperature map for biome classification.
///
/// Uses noise to create a smooth temperature gradient across the world.
/// Higher values represent warmer regions, lower values represent colder regions.
#[derive(Debug, Clone)]
pub struct TemperatureMapStage {
    stage_id: StageId,
    output: ResourceKey,
    scale: f64,
    domain: SeedDomain,
    products: [ResourceProduct; 1],
}

impl TemperatureMapStage {
    /// Creates a temperature map stage.
    pub fn new(
        stage_id: StageId,
        output: ResourceKey,
        scale: f64,
        domain: SeedDomain,
    ) -> Result<Self, StageError> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(StageError::new(
                "temperature scale must be finite and greater than zero",
            ));
        }

        Ok(Self {
            stage_id,
            output,
            scale,
            domain,
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
        })
    }

    /// Returns the resource key this stage creates.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns the configured scale.
    pub const fn scale(&self) -> f64 {
        self.scale
    }

    /// Returns the domain used to derive this stage's sampler seed.
    pub const fn domain(&self) -> SeedDomain {
        self.domain
    }
}

impl TerrainStage for TemperatureMapStage {
    fn id(&self) -> StageId {
        self.stage_id
    }

    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        let region = context
            .region()
            .as_region2()
            .ok_or_else(|| StageError::new("temperature stage requires a 2D generation region"))?;

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create temperature map at {:?}: resource already exists",
                self.output
            )));
        }

        let extent = region.point_sample_extent();

        let derived_seed = context.derive_seed(self.domain);
        let noise = NoiseAlgorithm::Simplex.create_2d(derived_seed);

        let mut values = Vec::with_capacity((extent.width() * extent.height()) as usize);

        let transform = region.transform();

        for y in 0..extent.height() {
            for x in 0..extent.width() {
                let world_position = transform.position_at(f64::from(x), f64::from(y));

                let sample_x = world_position.x * self.scale;
                let sample_y = world_position.z * self.scale;

                let temperature = ((Noise2D::sample(&noise, sample_x, sample_y) as f64 + 1.0)
                    * 0.5)
                    .clamp(0.0, 1.0);

                values.push(temperature as f32);
            }
        }

        let height_field = HeightField::from_values(
            extent,
            values,
            region.transform(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .map_err(|error| StageError::new(format!("failed to create temperature map: {error}")))?;

        resources.insert(self.output, TerrainResource::HeightField(height_field));

        Ok(())
    }
}
