//! Height-field noise stage implementation.

use terrakit_algorithms::{
    AlgorithmError,
    noise::{FractalSettings, NoiseAlgorithm, fractal_sample_2d},
};
use terrakit_core::{SeedDomain, TerrainResource};
use terrakit_pipeline::{
    ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement, ResourceSet,
    StageContext, StageError, StageId, TerrainStage,
};

const STAGE_NAME: &str = "noise_height";

/// How sampled height noise is combined with each existing height sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeightNoiseMode {
    /// Adds sampled noise to the existing height.
    #[default]
    Add,

    /// Replaces the existing height with sampled noise.
    Replace,

    /// Multiplies the existing height by sampled noise.
    Multiply,
}

impl HeightNoiseMode {
    fn combine(self, existing: f32, sampled: f32) -> f32 {
        match self {
            Self::Add => existing + sampled,
            Self::Replace => sampled,
            Self::Multiply => existing * sampled,
        }
    }
}

/// Terrain stage that applies configurable built-in 2D fractal noise to a height field.
///
/// This built-in stage targets TerraKit's standard horizontal XZ heightfields.
/// It samples noise using each height sample's world-space base position:
/// noise X is TerraKit world X, and noise Y is TerraKit world Z. Coordinates do
/// not restart at local zero for each region, so independently generated
/// neighbouring regions share identical boundary inputs.
#[derive(Debug, Clone)]
pub struct NoiseHeightStage {
    stage_id: StageId,
    input: ResourceKey,
    output: ResourceKey,
    algorithm: NoiseAlgorithm,
    settings: FractalSettings,
    domain: SeedDomain,
    mode: HeightNoiseMode,
    requirements: [ResourceRequirement; 1],
    products: [ResourceProduct; 1],
}

impl NoiseHeightStage {
    /// Creates a height-noise stage using a built-in TerraKit noise algorithm.
    ///
    /// The concrete noise sampler is created at execution time from the
    /// pipeline's [`StageContext::seed`] derived through `domain`.
    pub fn new(
        stage_id: StageId,
        input: ResourceKey,
        output: ResourceKey,
        algorithm: NoiseAlgorithm,
        settings: FractalSettings,
        domain: SeedDomain,
        mode: HeightNoiseMode,
    ) -> Result<Self, StageError> {
        let settings = settings
            .validate()
            .map_err(|error| algorithm_stage_error("invalid height noise settings", error))?;

        Ok(Self {
            stage_id,
            input,
            output,
            algorithm,
            settings,
            domain,
            mode,
            requirements: [ResourceRequirement::required(
                input,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
        })
    }

    /// Returns the height-field key this stage reads.
    pub const fn input(&self) -> ResourceKey {
        self.input
    }

    /// Returns the height-field key this stage creates.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns the configured built-in noise algorithm.
    pub const fn algorithm(&self) -> NoiseAlgorithm {
        self.algorithm
    }

    /// Returns the validated fractal sampling settings.
    pub const fn settings(&self) -> FractalSettings {
        self.settings
    }

    /// Returns the domain used to derive this stage's sampler seed.
    pub const fn domain(&self) -> SeedDomain {
        self.domain
    }

    /// Returns the configured height combination mode.
    pub const fn mode(&self) -> HeightNoiseMode {
        self.mode
    }
}

impl TerrainStage for NoiseHeightStage {
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
        let _region = context
            .region()
            .as_region2()
            .ok_or_else(|| StageError::new("noise height stage requires a 2D generation region"))?;
        let derived_seed = context.derive_seed(self.domain);
        let noise = self.algorithm.create_2d(derived_seed);
        if self.input == self.output {
            return Err(StageError::new(
                "height noise input and output keys must be distinct",
            ));
        }

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create height noise output at {:?}: resource already exists",
                self.output
            )));
        }

        let Some(source_height_field) = resources.height_field(self.input) else {
            return Err(StageError::new(format!(
                "height noise input {:?} is missing or is not a height field",
                self.input
            )));
        };
        let mut output_height_field = source_height_field.clone();

        for y in 0..output_height_field.height() {
            for x in 0..output_height_field.width() {
                let Some(existing) = output_height_field.get(x, y) else {
                    return Err(StageError::new(format!(
                        "missing height sample at ({x}, {y})"
                    )));
                };
                let Some(base_position) = output_height_field.base_position(x, y) else {
                    return Err(StageError::new(format!(
                        "missing height sample position at ({x}, {y})"
                    )));
                };

                let sampled =
                    fractal_sample_2d(&noise, self.settings, base_position.x, base_position.z)
                        .map_err(|error| {
                            algorithm_stage_error(
                                &format!("failed to sample height noise at ({x}, {y})"),
                                error,
                            )
                        })?;
                let next = self.mode.combine(existing, sampled);

                if !next.is_finite() {
                    return Err(StageError::new(format!(
                        "height noise produced a non-finite sample at ({x}, {y})"
                    )));
                }

                output_height_field.try_set(x, y, next).map_err(|error| {
                    StageError::new(format!(
                        "failed to write height sample at ({x}, {y}): {error}"
                    ))
                })?;
            }
        }

        resources.insert(
            self.output,
            TerrainResource::HeightField(output_height_field),
        );

        Ok(())
    }
}

fn algorithm_stage_error(operation: &str, error: AlgorithmError) -> StageError {
    StageError::new(format!("{operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrakit_algorithms::noise::fractal_sample_2d;
    use terrakit_core::{
        Extent2, Extent3, GenerationRegion, GenerationSeed, HeightField, LodLevel, RegionCoord2,
        RegionCoord3, RegionLayout2, RegionLayout3, RegionRequest2, RegionRequest3, SamplingDomain,
        TerrainResource, Vector2F64, Vector3F64,
    };
    use terrakit_pipeline::{ResourceKind, TerrainPipeline};

    fn cell_extent() -> Extent2 {
        Extent2::try_new(8, 6).unwrap()
    }

    fn layout() -> RegionLayout2 {
        RegionLayout2::new(cell_extent(), Vector2F64::new(1.0, 1.0)).unwrap()
    }

    fn settings() -> FractalSettings {
        FractalSettings {
            octaves: 3,
            frequency: 0.37,
            lacunarity: 2.0,
            persistence: 0.55,
            amplitude: 1.0,
            normalize: true,
        }
    }

    const SOURCE_HEIGHT: ResourceKey = ResourceKey(100);
    const NOISY_HEIGHT: ResourceKey = ResourceKey(101);
    const SECOND_NOISY_HEIGHT: ResourceKey = ResourceKey(102);

    fn context(seed: u64, layout: RegionLayout2, coordinate: RegionCoord2) -> StageContext {
        let region = GenerationRegion::Region2(
            layout
                .resolve(RegionRequest2::new(coordinate, LodLevel::HIGHEST))
                .unwrap(),
        );

        StageContext::from_u64(seed, region)
    }

    fn context3(seed: u64) -> StageContext {
        let layout = RegionLayout3::new(
            Extent3::try_new(2, 2, 2).unwrap(),
            Vector3F64::new(1.0, 1.0, 1.0),
        )
        .unwrap();
        let region = GenerationRegion::Region3(
            layout
                .resolve(RegionRequest3::new(RegionCoord3::ZERO, LodLevel::HIGHEST))
                .unwrap(),
        );

        StageContext::from_u64(seed, region)
    }

    fn height_field(value: f32, context: &StageContext) -> HeightField {
        let region = context.region2().unwrap();

        HeightField::filled(
            region.point_sample_extent(),
            value,
            region.transform(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .unwrap()
    }

    fn run_pipeline_for(
        seed: u64,
        layout: RegionLayout2,
        coordinate: RegionCoord2,
        domain: SeedDomain,
        mode: HeightNoiseMode,
    ) -> HeightField {
        let mut pipeline = TerrainPipeline::new();
        pipeline
            .add_stage(crate::FlatHeightStage::standard(StageId(1), SOURCE_HEIGHT, 2.0).unwrap());
        pipeline.add_stage(
            NoiseHeightStage::new(
                StageId(2),
                SOURCE_HEIGHT,
                ResourceKey::HEIGHT,
                NoiseAlgorithm::Value,
                settings(),
                domain,
                mode,
            )
            .unwrap(),
        );

        let mut resources = ResourceSet::new();
        pipeline
            .execute(&context(seed, layout, coordinate), &mut resources)
            .unwrap();

        resources
            .height_field(ResourceKey::HEIGHT)
            .cloned()
            .unwrap()
    }

    fn run_pipeline(seed: u64, domain: SeedDomain, mode: HeightNoiseMode) -> HeightField {
        run_pipeline_for(seed, layout(), RegionCoord2::ZERO, domain, mode)
    }

    #[test]
    fn declares_readonly_requirement_and_create_product() {
        let stage = NoiseHeightStage::new(
            StageId(9),
            SOURCE_HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(99),
            HeightNoiseMode::Add,
        )
        .unwrap();

        assert_eq!(stage.id(), StageId(9));
        assert_eq!(stage.name(), "noise_height");
        assert_eq!(stage.input(), SOURCE_HEIGHT);
        assert_eq!(stage.output(), NOISY_HEIGHT);
        assert_eq!(
            stage.requirements(),
            &[ResourceRequirement::required(
                SOURCE_HEIGHT,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )]
        );
        assert_eq!(
            stage.products(),
            &[ResourceProduct::create(
                NOISY_HEIGHT,
                ResourceKind::HeightField,
            )]
        );
    }

    #[test]
    fn same_seed_and_domain_are_deterministic() {
        let first = run_pipeline(123, SeedDomain::new(7), HeightNoiseMode::Add);
        let second = run_pipeline(123, SeedDomain::new(7), HeightNoiseMode::Add);

        assert_eq!(first.values(), second.values());
    }

    #[test]
    fn different_generation_seeds_change_output() {
        let first = run_pipeline(123, SeedDomain::new(7), HeightNoiseMode::Add);
        let second = run_pipeline(124, SeedDomain::new(7), HeightNoiseMode::Add);

        assert!(
            first
                .values()
                .iter()
                .zip(second.values())
                .any(|(first, second)| first != second)
        );
    }

    #[test]
    fn different_domains_change_output() {
        let first = run_pipeline(123, SeedDomain::new(7), HeightNoiseMode::Add);
        let second = run_pipeline(123, SeedDomain::new(8), HeightNoiseMode::Add);

        assert!(
            first
                .values()
                .iter()
                .zip(second.values())
                .any(|(first, second)| first != second)
        );
    }

    #[test]
    fn zero_amplitude_additive_noise_keeps_heightfield_unchanged() {
        let mut settings = settings();
        settings.amplitude = 0.0;
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings,
            SeedDomain::new(7),
            HeightNoiseMode::Add,
        )
        .unwrap();
        let context = context(123, layout(), RegionCoord2::ZERO);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(3.25, &context)),
        );

        stage.execute(&context, &mut resources).unwrap();

        let source = resources.height_field(ResourceKey::HEIGHT).unwrap();
        let output = resources.height_field(NOISY_HEIGHT).unwrap();
        assert!(source.values().iter().all(|value| *value == 3.25));
        assert!(output.values().iter().all(|value| *value == 3.25));
    }

    #[test]
    fn input_and_output_keys_must_be_distinct() {
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            ResourceKey::HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(7),
            HeightNoiseMode::Add,
        )
        .unwrap();
        let context = context(123, layout(), RegionCoord2::ZERO);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(3.25, &context)),
        );

        let error = stage.execute(&context, &mut resources).unwrap_err();

        assert!(error.message().contains("must be distinct"));
    }

    #[test]
    fn occupied_output_is_rejected() {
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(7),
            HeightNoiseMode::Add,
        )
        .unwrap();
        let context = context(123, layout(), RegionCoord2::ZERO);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(3.25, &context)),
        );
        resources.insert(
            NOISY_HEIGHT,
            TerrainResource::HeightField(height_field(0.0, &context)),
        );

        let error = stage.execute(&context, &mut resources).unwrap_err();

        assert!(error.message().contains("resource already exists"));
    }

    #[test]
    fn source_heightfield_remains_unchanged_when_output_is_modified() {
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(7),
            HeightNoiseMode::Add,
        )
        .unwrap();
        let context = context(123, layout(), RegionCoord2::ZERO);
        let original = height_field(3.25, &context);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(original.clone()),
        );

        stage.execute(&context, &mut resources).unwrap();

        let source = resources.height_field(ResourceKey::HEIGHT).unwrap();
        let output = resources.height_field(NOISY_HEIGHT).unwrap();
        assert_eq!(source, &original);
        assert!(
            output
                .values()
                .iter()
                .zip(original.values())
                .any(|(output, original)| output != original)
        );
    }

    #[test]
    fn replace_mode_uses_sampled_noise_value() {
        let domain = SeedDomain::new(77);
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            domain,
            HeightNoiseMode::Replace,
        )
        .unwrap();
        let context = context(5, layout(), RegionCoord2::ZERO);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(10.0, &context)),
        );

        stage.execute(&context, &mut resources).unwrap();

        let noise = NoiseAlgorithm::Value.create_2d(GenerationSeed::new(5).derive(domain));
        let expected = fractal_sample_2d(&noise, settings(), 1.0, 1.0).unwrap();
        let height_field = resources.height_field(NOISY_HEIGHT).unwrap();

        assert_eq!(height_field.get(1, 1), Some(expected));
    }

    #[test]
    fn multiply_mode_multiplies_by_sampled_noise_value() {
        let domain = SeedDomain::new(78);
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            domain,
            HeightNoiseMode::Multiply,
        )
        .unwrap();
        let context = context(5, layout(), RegionCoord2::ZERO);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(2.0, &context)),
        );

        stage.execute(&context, &mut resources).unwrap();

        let noise = NoiseAlgorithm::Value.create_2d(GenerationSeed::new(5).derive(domain));
        let sampled = fractal_sample_2d(&noise, settings(), 1.0, 1.0).unwrap();
        let height_field = resources.height_field(NOISY_HEIGHT).unwrap();

        assert_eq!(height_field.get(1, 1), Some(2.0 * sampled));
    }

    #[test]
    fn sampling_uses_transform_scale() {
        let layout = RegionLayout2::new(cell_extent(), Vector2F64::new(2.0, 2.0)).unwrap();
        let domain = SeedDomain::new(17);
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            domain,
            HeightNoiseMode::Replace,
        )
        .unwrap();
        let context = context(5, layout, RegionCoord2::ZERO);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(0.0, &context)),
        );

        stage.execute(&context, &mut resources).unwrap();

        let noise = NoiseAlgorithm::Value.create_2d(GenerationSeed::new(5).derive(domain));
        let expected = fractal_sample_2d(&noise, settings(), 2.0, 2.0).unwrap();
        let height_field = resources.height_field(NOISY_HEIGHT).unwrap();

        assert_eq!(height_field.get(1, 1), Some(expected));
    }

    #[test]
    fn noise_coordinates_use_region_spatial_placement() {
        let layout = layout();
        let coordinate = RegionCoord2::new(1, 0);
        let domain = SeedDomain::new(18);
        let context = context(5, layout, coordinate);
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            domain,
            HeightNoiseMode::Replace,
        )
        .unwrap();
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(0.0, &context)),
        );

        stage.execute(&context, &mut resources).unwrap();

        let noise = NoiseAlgorithm::Value.create_2d(GenerationSeed::new(5).derive(domain));
        let expected = fractal_sample_2d(&noise, settings(), 8.0, 0.0).unwrap();
        let height_field = resources.height_field(NOISY_HEIGHT).unwrap();

        assert_eq!(height_field.get(0, 0), Some(expected));
    }

    #[test]
    fn adjacent_regions_share_exact_boundary_values() {
        let layout = layout();
        let left = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(0, 0),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let right = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(1, 0),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let edge_x = cell_extent().width_usize();

        for y in 0..left.height() {
            assert_eq!(left.get(edge_x, y), right.get(0, y));
        }
    }

    #[test]
    fn negative_coordinate_regions_remain_seamless() {
        let layout = layout();
        let left = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(-1, 0),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let right = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(0, 0),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let edge_x = cell_extent().width_usize();

        for y in 0..left.height() {
            assert_eq!(left.get(edge_x, y), right.get(0, y));
        }
    }

    #[test]
    fn four_regions_share_the_same_corner_sample() {
        let layout = layout();
        let base = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(0, 0),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let right = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(1, 0),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let above = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(0, 1),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let diagonal = run_pipeline_for(
            123,
            layout,
            RegionCoord2::new(1, 1),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let edge_x = cell_extent().width_usize();
        let edge_y = cell_extent().height_usize();
        let corner = base.get(edge_x, edge_y);

        assert_eq!(corner, right.get(0, edge_y));
        assert_eq!(corner, above.get(edge_x, 0));
        assert_eq!(corner, diagonal.get(0, 0));
    }

    #[test]
    fn same_region_generated_through_separate_pipeline_instances_is_identical() {
        let first = run_pipeline_for(
            123,
            layout(),
            RegionCoord2::new(2, -1),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let second = run_pipeline_for(
            123,
            layout(),
            RegionCoord2::new(2, -1),
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );

        assert_eq!(first.values(), second.values());
        assert_eq!(content_hash(&first), content_hash(&second));
    }

    #[test]
    fn content_hash_changes_for_different_seeds() {
        let first = run_pipeline_for(
            123,
            layout(),
            RegionCoord2::ZERO,
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );
        let second = run_pipeline_for(
            124,
            layout(),
            RegionCoord2::ZERO,
            SeedDomain::new(7),
            HeightNoiseMode::Replace,
        );

        assert_ne!(content_hash(&first), content_hash(&second));
    }

    #[test]
    fn branch_outputs_are_independent_of_branch_execution_order() {
        let (source_ab, noise_a_ab, noise_b_ab) = run_branch_pipeline(true);
        let (source_ba, noise_a_ba, noise_b_ba) = run_branch_pipeline(false);

        assert_eq!(source_ab, source_ba);
        assert_eq!(noise_a_ab, noise_a_ba);
        assert_eq!(noise_b_ab, noise_b_ba);
        assert!(source_ab.values().iter().all(|value| *value == 2.0));
    }

    #[test]
    fn rejects_invalid_fractal_settings() {
        for invalid in [
            FractalSettings {
                octaves: 0,
                ..settings()
            },
            FractalSettings {
                frequency: f64::NAN,
                ..settings()
            },
            FractalSettings {
                lacunarity: 0.0,
                ..settings()
            },
            FractalSettings {
                persistence: -0.1,
                ..settings()
            },
            FractalSettings {
                amplitude: f32::INFINITY,
                ..settings()
            },
        ] {
            assert!(
                NoiseHeightStage::new(
                    StageId(2),
                    ResourceKey::HEIGHT,
                    NOISY_HEIGHT,
                    NoiseAlgorithm::Value,
                    invalid,
                    SeedDomain::new(1),
                    HeightNoiseMode::Add,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn valid_configuration_keeps_all_outputs_finite() {
        let height_field = run_pipeline(123, SeedDomain::new(7), HeightNoiseMode::Add);

        assert!(height_field.values().iter().all(|value| value.is_finite()));
    }

    #[test]
    fn direct_execution_reports_missing_resource() {
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(7),
            HeightNoiseMode::Add,
        )
        .unwrap();
        let mut resources = ResourceSet::new();

        let error = stage
            .execute(&context(123, layout(), RegionCoord2::ZERO), &mut resources)
            .unwrap_err();

        assert!(error.message().contains("is missing"));
    }

    #[test]
    fn rejects_3d_generation_context() {
        let mut stage = NoiseHeightStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(7),
            HeightNoiseMode::Add,
        )
        .unwrap();
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(HeightField::from_dimensions(1, 1).unwrap()),
        );

        let error = stage.execute(&context3(123), &mut resources).unwrap_err();

        assert!(error.message().contains("requires a 2D generation region"));
    }

    fn content_hash(height_field: &HeightField) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;

        for sample in height_field.values() {
            for byte in sample.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }

        hash
    }

    fn run_branch_pipeline(a_then_b: bool) -> (HeightField, HeightField, HeightField) {
        let mut pipeline = TerrainPipeline::new();
        pipeline
            .add_stage(crate::FlatHeightStage::standard(StageId(1), SOURCE_HEIGHT, 2.0).unwrap());
        let noise_a = NoiseHeightStage::new(
            StageId(2),
            SOURCE_HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(7),
            HeightNoiseMode::Add,
        )
        .unwrap();
        let noise_b = NoiseHeightStage::new(
            StageId(3),
            SOURCE_HEIGHT,
            SECOND_NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(8),
            HeightNoiseMode::Multiply,
        )
        .unwrap();

        if a_then_b {
            pipeline.add_stage(noise_a);
            pipeline.add_stage(noise_b);
        } else {
            pipeline.add_stage(noise_b);
            pipeline.add_stage(noise_a);
        }

        let mut resources = ResourceSet::new();
        pipeline
            .execute(&context(123, layout(), RegionCoord2::ZERO), &mut resources)
            .unwrap();

        (
            resources.height_field(SOURCE_HEIGHT).cloned().unwrap(),
            resources.height_field(NOISY_HEIGHT).cloned().unwrap(),
            resources
                .height_field(SECOND_NOISY_HEIGHT)
                .cloned()
                .unwrap(),
        )
    }
}
