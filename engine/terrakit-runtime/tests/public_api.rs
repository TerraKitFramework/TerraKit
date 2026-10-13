use terrakit_core::{
    Extent2, GenerationSeed, HeightField, LodLevel, RegionCoord2, RegionLayout2, SamplingDomain,
    TerrainResource, Vector2F64, Vector3F64,
};
use terrakit_pipeline::{
    ResourceKey, ResourceKind, ResourceProduct, ResourceSet, StageContext, StageError, StageId,
    TerrainPipeline, TerrainStage,
};
use terrakit_runtime::{GenerationRequest, TerrainRuntime};

#[test]
fn generation_result_exposes_owned_resources_by_shared_reference() {
    let layout =
        RegionLayout2::new(Extent2::try_new(4, 4).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(ProduceHeightStage::new(ResourceKey::HEIGHT));
    let mut runtime = TerrainRuntime::new(layout, pipeline);
    let request = GenerationRequest::region2(
        GenerationSeed::new(9),
        RegionCoord2::new(2, -3),
        LodLevel::HIGHEST,
    );

    let result = runtime.generate(request).unwrap();
    let resources = result.resources();

    assert_eq!(result.request(), request);
    assert!(resources.height_field(ResourceKey::HEIGHT).is_some());

    drop(runtime);

    assert!(
        result
            .resources()
            .height_field(ResourceKey::HEIGHT)
            .is_some()
    );

    let owned_resources = result.into_resources();
    assert!(owned_resources.height_field(ResourceKey::HEIGHT).is_some());
}

struct ProduceHeightStage {
    output: ResourceKey,
    products: [ResourceProduct; 1],
}

impl ProduceHeightStage {
    fn new(output: ResourceKey) -> Self {
        Self {
            output,
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
        }
    }
}

impl TerrainStage for ProduceHeightStage {
    fn id(&self) -> StageId {
        StageId(1)
    }

    fn name(&self) -> &str {
        "produce_height"
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        let region = context.region2()?;
        let height = HeightField::filled(
            region.point_sample_extent(),
            2.5,
            region.transform(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .map_err(StageError::from)?;

        resources.insert(self.output, TerrainResource::HeightField(height));

        Ok(())
    }
}
