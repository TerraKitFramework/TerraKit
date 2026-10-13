//! Flat height-field stage implementation.

use terrakit_core::{CoreError, HeightField, SamplingDomain, TerrainResource, Vector3F64};
use terrakit_pipeline::{
    ResourceKey, ResourceKind, ResourceProduct, ResourceSet, StageContext, StageError, StageId,
    TerrainStage,
};

const STAGE_NAME: &str = "flat_height";

/// Terrain stage that creates a canonical height field filled with one value.
///
/// The stage declares no requirements and creates its configured output key as
/// a `HeightField` resource. Direct execution rejects an already occupied
/// output key, matching the pipeline's create-only product declaration.
#[derive(Debug, Clone)]
pub struct FlatHeightStage {
    stage_id: StageId,
    output: ResourceKey,
    elevation: f32,
    height_axis: Vector3F64,
    products: [ResourceProduct; 1],
}

impl FlatHeightStage {
    /// Creates a flat height-field stage from region-independent settings.
    pub fn new(
        stage_id: StageId,
        output: ResourceKey,
        elevation: f32,
        height_axis: Vector3F64,
    ) -> Result<Self, StageError> {
        validate_configuration(elevation, height_axis)?;

        Ok(Self {
            stage_id,
            output,
            elevation,
            height_axis,
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
        })
    }

    /// Creates a conventional XZ-plane height field with positive Y elevation.
    pub fn standard(
        stage_id: StageId,
        output: ResourceKey,
        elevation: f32,
    ) -> Result<Self, StageError> {
        Self::new(stage_id, output, elevation, Vector3F64::Y)
    }

    /// Returns the resource key this stage creates.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns the configured constant elevation.
    pub const fn elevation(&self) -> f32 {
        self.elevation
    }

    /// Returns the configured height axis.
    pub const fn height_axis(&self) -> Vector3F64 {
        self.height_axis
    }
}

impl TerrainStage for FlatHeightStage {
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
            .ok_or_else(|| StageError::new("flat height stage requires a 2D generation region"))?;

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create flat height field at {:?}: resource already exists",
                self.output
            )));
        }

        let height_field = HeightField::filled(
            region.point_sample_extent(),
            self.elevation,
            region.transform(),
            SamplingDomain::Points,
            self.height_axis,
        )
        .map_err(|error| core_stage_error("failed to create flat height field", error))?;

        resources.insert(self.output, TerrainResource::HeightField(height_field));

        Ok(())
    }
}

fn validate_configuration(elevation: f32, height_axis: Vector3F64) -> Result<(), StageError> {
    if !elevation.is_finite() {
        return Err(StageError::new("flat height elevation must be finite"));
    }

    if !height_axis.is_finite() || height_axis.length_squared() == 0.0 {
        return Err(StageError::new(
            "flat height axis must be finite and non-zero",
        ));
    }

    Ok(())
}

fn core_stage_error(operation: &str, error: CoreError) -> StageError {
    StageError::new(format!("{operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrakit_core::{
        Extent2, Extent3, GenerationRegion, GenerationSeed, LodLevel, RegionCoord2, RegionCoord3,
        RegionLayout2, RegionLayout3, RegionRequest2, RegionRequest3, Vector2F64,
    };
    use terrakit_pipeline::ResourceKind;

    fn layout2(width: u32, height: u32, spacing_x: f64, spacing_y: f64) -> RegionLayout2 {
        RegionLayout2::new(
            Extent2::try_new(width, height).unwrap(),
            Vector2F64::new(spacing_x, spacing_y),
        )
        .unwrap()
    }

    fn context2(layout: RegionLayout2, coordinate: RegionCoord2) -> StageContext {
        let region = GenerationRegion::Region2(
            layout
                .resolve(RegionRequest2::new(coordinate, LodLevel::HIGHEST))
                .unwrap(),
        );

        StageContext::new(GenerationSeed::new(123), region)
    }

    fn context3() -> StageContext {
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

        StageContext::new(GenerationSeed::new(123), region)
    }

    #[test]
    fn declares_no_requirements_and_one_heightfield_product() {
        let stage = FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, 4.0).unwrap();

        assert!(stage.requirements().is_empty());
        assert_eq!(
            stage.products(),
            &[ResourceProduct::create(
                ResourceKey::HEIGHT,
                ResourceKind::HeightField,
            )]
        );
    }

    #[test]
    fn refuses_invalid_elevation() {
        let error =
            FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, f32::NAN).unwrap_err();

        assert!(error.message().contains("elevation must be finite"));
    }

    #[test]
    fn refuses_invalid_height_axis() {
        let error = FlatHeightStage::new(StageId(1), ResourceKey::HEIGHT, 0.0, Vector3F64::ZERO)
            .unwrap_err();

        assert!(error.message().contains("axis must be finite and non-zero"));
    }

    #[test]
    fn uses_region_point_sample_extent_and_transform() {
        let layout = layout2(3, 2, 2.0, 4.0);
        let context = context2(layout, RegionCoord2::new(2, -1));
        let region = *context.region2().unwrap();
        let height_axis = Vector3F64::new(0.0, 2.0, 0.0);
        let mut stage =
            FlatHeightStage::new(StageId(7), ResourceKey::HEIGHT, 12.5, height_axis).unwrap();
        let mut resources = ResourceSet::new();

        stage.execute(&context, &mut resources).unwrap();

        let Some(TerrainResource::HeightField(height_field)) = resources.get(ResourceKey::HEIGHT)
        else {
            panic!("expected canonical height field");
        };

        assert_eq!(height_field.extent(), region.point_sample_extent());
        assert!(height_field.values().iter().all(|value| *value == 12.5));
        assert_eq!(height_field.transform(), region.transform());
        assert_eq!(height_field.height_axis(), height_axis);
    }

    #[test]
    fn same_stage_instance_can_generate_different_region_coordinates() {
        let layout = layout2(3, 2, 1.0, 1.0);
        let mut stage = FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, 4.0).unwrap();
        let mut first_resources = ResourceSet::new();
        let mut second_resources = ResourceSet::new();

        stage
            .execute(
                &context2(layout, RegionCoord2::new(0, 0)),
                &mut first_resources,
            )
            .unwrap();
        stage
            .execute(
                &context2(layout, RegionCoord2::new(1, 0)),
                &mut second_resources,
            )
            .unwrap();

        let first = first_resources.height_field(ResourceKey::HEIGHT).unwrap();
        let second = second_resources.height_field(ResourceKey::HEIGHT).unwrap();

        assert_eq!(first.extent(), second.extent());
        assert_eq!(first.transform().origin(), Vector3F64::ZERO);
        assert_eq!(second.transform().origin(), Vector3F64::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn stage_does_not_store_region_specific_dimensions() {
        let mut stage = FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, 4.0).unwrap();
        let mut small_resources = ResourceSet::new();
        let mut large_resources = ResourceSet::new();

        stage
            .execute(
                &context2(layout2(3, 2, 1.0, 1.0), RegionCoord2::ZERO),
                &mut small_resources,
            )
            .unwrap();
        stage
            .execute(
                &context2(layout2(5, 4, 1.0, 1.0), RegionCoord2::ZERO),
                &mut large_resources,
            )
            .unwrap();

        assert_eq!(
            small_resources
                .height_field(ResourceKey::HEIGHT)
                .unwrap()
                .extent(),
            Extent2::try_new(4, 3).unwrap()
        );
        assert_eq!(
            large_resources
                .height_field(ResourceKey::HEIGHT)
                .unwrap()
                .extent(),
            Extent2::try_new(6, 5).unwrap()
        );
    }

    #[test]
    fn rejects_3d_generation_context() {
        let mut stage = FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, 4.0).unwrap();
        let mut resources = ResourceSet::new();

        let error = stage.execute(&context3(), &mut resources).unwrap_err();

        assert!(error.message().contains("requires a 2D generation region"));
    }

    #[test]
    fn direct_execution_refuses_to_overwrite_existing_resources() {
        let mut stage = FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, 4.0).unwrap();
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(HeightField::from_dimensions(1, 1).unwrap()),
        );

        let error = stage
            .execute(
                &context2(layout2(1, 1, 1.0, 1.0), RegionCoord2::ZERO),
                &mut resources,
            )
            .unwrap_err();

        assert!(error.message().contains("resource already exists"));
    }
}
