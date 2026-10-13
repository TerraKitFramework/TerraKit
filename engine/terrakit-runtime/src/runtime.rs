//! Synchronous headless runtime implementation.

use terrakit_core::GenerationRegion;
use terrakit_pipeline::{ResourceSet, StageContext, TerrainPipeline};

use crate::{GenerationLayout, GenerationRequest, GenerationResult, RuntimeError};

/// Synchronous headless runtime for executing one configured terrain pipeline.
///
/// A runtime owns one immutable generation layout and one mutable terrain
/// pipeline. Pipeline stage state persists across generation calls, while each
/// call receives a fresh `ResourceSet` that becomes owned by the returned
/// [`GenerationResult`] on success.
///
/// A `TerrainRuntime` executes one request at a time. For parallel generation,
/// create one runtime instance per worker thread or node.
pub struct TerrainRuntime {
    layout: GenerationLayout,
    pipeline: TerrainPipeline,
}

impl TerrainRuntime {
    /// Creates a runtime from an already configured layout and terrain pipeline.
    pub fn new(layout: impl Into<GenerationLayout>, pipeline: TerrainPipeline) -> Self {
        Self {
            layout: layout.into(),
            pipeline,
        }
    }

    /// Returns the immutable generation layout owned by this runtime.
    pub const fn layout(&self) -> &GenerationLayout {
        &self.layout
    }

    /// Returns the configured terrain pipeline.
    pub fn pipeline(&self) -> &TerrainPipeline {
        &self.pipeline
    }

    /// Returns mutable access to the configured terrain pipeline.
    ///
    /// This is an advanced Rust-host configuration hook and is not intended to
    /// be exposed directly through the first C ABI. Modifying the pipeline
    /// changes subsequent generation output; previously returned results remain
    /// independently owned and unchanged.
    pub fn pipeline_mut(&mut self) -> &mut TerrainPipeline {
        &mut self.pipeline
    }

    /// Consumes the runtime and returns its configured terrain pipeline.
    pub fn into_pipeline(self) -> TerrainPipeline {
        self.pipeline
    }

    /// Generates one region by resolving `request`, executing the pipeline, and returning owned resources.
    ///
    /// Every invocation starts with a fresh empty `ResourceSet`. Successful
    /// generation returns a complete [`GenerationResult`]; failed generation
    /// returns a [`RuntimeError`] and does not expose partially generated
    /// resources. Pipeline stage state and external side effects are not rolled
    /// back when a stage fails.
    pub fn generate(
        &mut self,
        request: GenerationRequest,
    ) -> Result<GenerationResult, RuntimeError> {
        if self.layout.kind() != request.kind() {
            return Err(RuntimeError::LayoutMismatch {
                configured: self.layout.kind(),
                requested: request.kind(),
            });
        }

        let region = self.resolve_request(request)?;
        let context = StageContext::new(request.seed(), region);
        let mut resources = ResourceSet::new();

        self.pipeline
            .execute(&context, &mut resources)
            .map_err(|source| RuntimeError::PipelineExecution { source })?;

        Ok(GenerationResult::new(request, region, resources))
    }

    fn resolve_request(
        &self,
        request: GenerationRequest,
    ) -> Result<GenerationRegion, RuntimeError> {
        match (&self.layout, request) {
            (GenerationLayout::Region2(layout), GenerationRequest::Region2 { region, .. }) => {
                let descriptor = layout
                    .resolve(region)
                    .map_err(|source| RuntimeError::RegionResolution { source })?;

                Ok(GenerationRegion::Region2(descriptor))
            }

            (GenerationLayout::Region3(layout), GenerationRequest::Region3 { region, .. }) => {
                let descriptor = layout
                    .resolve(region)
                    .map_err(|source| RuntimeError::RegionResolution { source })?;

                Ok(GenerationRegion::Region3(descriptor))
            }

            _ => Err(RuntimeError::LayoutMismatch {
                configured: self.layout.kind(),
                requested: request.kind(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
    };

    use terrakit_core::{
        CoreError, DensityField, Extent2, Extent3, GenerationSeed, HeightField, LodLevel,
        RegionCoord2, RegionCoord3, RegionLayout2, RegionLayout3, SamplingDomain, TerrainResource,
        Vector2F64, Vector3F64,
    };
    use terrakit_pipeline::{
        PipelineError, ResourceKey, ResourceKind, ResourceProduct, ResourceSet, StageContext,
        StageError, StageId, TerrainStage,
    };

    use super::*;

    fn layout2() -> RegionLayout2 {
        RegionLayout2::new(Extent2::try_new(4, 4).unwrap(), Vector2F64::new(1.0, 2.0)).unwrap()
    }

    fn layout3() -> RegionLayout3 {
        RegionLayout3::new(
            Extent3::try_new(4, 4, 4).unwrap(),
            Vector3F64::new(1.0, 2.0, 3.0),
        )
        .unwrap()
    }

    fn request2(
        seed: GenerationSeed,
        coordinate: RegionCoord2,
        lod: LodLevel,
    ) -> GenerationRequest {
        GenerationRequest::region2(seed, coordinate, lod)
    }

    fn request3(
        seed: GenerationSeed,
        coordinate: RegionCoord3,
        lod: LodLevel,
    ) -> GenerationRequest {
        GenerationRequest::region3(seed, coordinate, lod)
    }

    fn assert_send<T: Send>() {}

    #[test]
    fn runtime_is_send() {
        assert_send::<TerrainRuntime>();
    }

    #[test]
    fn two_dimensional_generation_resolves_request_and_returns_resources() {
        let seed = GenerationSeed::new(123);
        let coordinate = RegionCoord2::new(2, -1);
        let lod = LodLevel::new(1);
        let executed = Arc::new(AtomicBool::new(false));
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(InspectRegion2Stage::new(
            seed,
            coordinate,
            lod,
            Arc::clone(&executed),
        ));
        pipeline.add_stage(ProduceHeightFieldStage::new(ResourceKey::HEIGHT, 4.0));
        let mut runtime = TerrainRuntime::new(layout2(), pipeline);

        let result = runtime.generate(request2(seed, coordinate, lod)).unwrap();

        assert!(executed.load(Ordering::SeqCst));
        assert_eq!(result.seed(), seed);
        assert_eq!(result.lod(), lod);
        assert_eq!(result.kind(), crate::GenerationRegionKind::Region2);
        assert_eq!(result.region2().unwrap().coordinate(), coordinate);
        assert_eq!(result.region2().unwrap().lod(), lod);
        assert!(result.region3().is_none());
        assert!(
            result
                .resources()
                .height_field(ResourceKey::HEIGHT)
                .is_some()
        );
    }

    #[test]
    fn three_dimensional_generation_resolves_request_and_returns_resources() {
        let seed = GenerationSeed::new(456);
        let coordinate = RegionCoord3::new(-2, 1, 3);
        let lod = LodLevel::new(1);
        let executed = Arc::new(AtomicBool::new(false));
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(InspectRegion3Stage::new(
            seed,
            coordinate,
            lod,
            Arc::clone(&executed),
        ));
        pipeline.add_stage(ProduceDensityFieldStage::new(ResourceKey::DENSITY, -0.25));
        let mut runtime = TerrainRuntime::new(layout3(), pipeline);

        let result = runtime.generate(request3(seed, coordinate, lod)).unwrap();

        assert!(executed.load(Ordering::SeqCst));
        assert_eq!(result.seed(), seed);
        assert_eq!(result.lod(), lod);
        assert_eq!(result.kind(), crate::GenerationRegionKind::Region3);
        assert_eq!(result.region3().unwrap().coordinate(), coordinate);
        assert_eq!(result.region3().unwrap().lod(), lod);
        assert!(result.region2().is_none());
        assert!(
            result
                .resources()
                .density_field(ResourceKey::DENSITY)
                .is_some()
        );
    }

    #[test]
    fn mismatched_layout_and_request_returns_error_without_executing_pipeline() {
        let executed_2d_runtime = Arc::new(AtomicBool::new(false));
        let mut pipeline_2d = TerrainPipeline::new();
        pipeline_2d.add_stage(MarkerStage::new(Arc::clone(&executed_2d_runtime)));
        let mut runtime_2d = TerrainRuntime::new(layout2(), pipeline_2d);

        let error = runtime_2d
            .generate(request3(
                GenerationSeed::new(1),
                RegionCoord3::ZERO,
                LodLevel::HIGHEST,
            ))
            .unwrap_err();

        match error {
            RuntimeError::LayoutMismatch {
                configured,
                requested,
            } => {
                assert_eq!(configured, crate::GenerationRegionKind::Region2);
                assert_eq!(requested, crate::GenerationRegionKind::Region3);
            }
            other => panic!("expected layout mismatch, got {other:?}"),
        }
        assert!(!executed_2d_runtime.load(Ordering::SeqCst));

        let executed_3d_runtime = Arc::new(AtomicBool::new(false));
        let mut pipeline_3d = TerrainPipeline::new();
        pipeline_3d.add_stage(MarkerStage::new(Arc::clone(&executed_3d_runtime)));
        let mut runtime_3d = TerrainRuntime::new(layout3(), pipeline_3d);

        let error = runtime_3d
            .generate(request2(
                GenerationSeed::new(1),
                RegionCoord2::ZERO,
                LodLevel::HIGHEST,
            ))
            .unwrap_err();

        match error {
            RuntimeError::LayoutMismatch {
                configured,
                requested,
            } => {
                assert_eq!(configured, crate::GenerationRegionKind::Region3);
                assert_eq!(requested, crate::GenerationRegionKind::Region2);
            }
            other => panic!("expected layout mismatch, got {other:?}"),
        }
        assert!(!executed_3d_runtime.load(Ordering::SeqCst));
    }

    #[test]
    fn region_resolution_errors_are_wrapped_and_pipeline_does_not_execute() {
        let layout =
            RegionLayout2::new(Extent2::try_new(3, 4).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
        let executed = Arc::new(AtomicBool::new(false));
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(MarkerStage::new(Arc::clone(&executed)));
        let mut runtime = TerrainRuntime::new(layout, pipeline);

        let error = runtime
            .generate(request2(
                GenerationSeed::new(1),
                RegionCoord2::ZERO,
                LodLevel::new(1),
            ))
            .unwrap_err();

        match &error {
            RuntimeError::RegionResolution { source } => {
                assert!(matches!(source, CoreError::LodNotDivisible { .. }));
            }
            other => panic!("expected region resolution error, got {other:?}"),
        }
        assert!(Error::source(&error).is_some());
        assert!(!executed.load(Ordering::SeqCst));
    }

    #[test]
    fn pipeline_errors_are_wrapped_and_exposed_as_sources() {
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(FailingStage);
        let mut runtime = TerrainRuntime::new(layout2(), pipeline);

        let error = runtime
            .generate(request2(
                GenerationSeed::new(1),
                RegionCoord2::ZERO,
                LodLevel::HIGHEST,
            ))
            .unwrap_err();

        match &error {
            RuntimeError::PipelineExecution { source } => {
                assert!(matches!(source, PipelineError::StageFailed { .. }));
            }
            other => panic!("expected pipeline execution error, got {other:?}"),
        }
        assert!(
            Error::source(&error)
                .unwrap()
                .to_string()
                .contains("stage 'failing'")
        );
    }

    #[test]
    fn result_resources_can_outlive_the_runtime() {
        let result = {
            let mut pipeline = TerrainPipeline::new();
            pipeline.add_stage(ProduceHeightFieldStage::new(ResourceKey::HEIGHT, 1.0));
            let mut runtime = TerrainRuntime::new(layout2(), pipeline);

            runtime
                .generate(request2(
                    GenerationSeed::new(1),
                    RegionCoord2::ZERO,
                    LodLevel::HIGHEST,
                ))
                .unwrap()
        };

        assert!(result.resources().contains(ResourceKey::HEIGHT));
        assert!(
            result
                .resources()
                .height_field(ResourceKey::HEIGHT)
                .is_some()
        );
    }

    #[test]
    fn multiple_outstanding_results_remain_independent() {
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(ProduceHeightFieldStage::new(ResourceKey::HEIGHT, 2.0));
        let mut runtime = TerrainRuntime::new(layout2(), pipeline);

        let first = runtime
            .generate(request2(
                GenerationSeed::new(1),
                RegionCoord2::new(0, 0),
                LodLevel::HIGHEST,
            ))
            .unwrap();
        let second = runtime
            .generate(request2(
                GenerationSeed::new(1),
                RegionCoord2::new(1, 0),
                LodLevel::HIGHEST,
            ))
            .unwrap();

        assert!(
            first
                .resources()
                .height_field(ResourceKey::HEIGHT)
                .is_some()
        );
        assert!(
            second
                .resources()
                .height_field(ResourceKey::HEIGHT)
                .is_some()
        );
        assert_ne!(
            first.region2().unwrap().coordinate(),
            second.region2().unwrap().coordinate()
        );
        assert_ne!(
            first
                .resources()
                .height_field(ResourceKey::HEIGHT)
                .unwrap()
                .transform()
                .origin(),
            second
                .resources()
                .height_field(ResourceKey::HEIGHT)
                .unwrap()
                .transform()
                .origin()
        );

        drop(first);

        assert!(
            second
                .resources()
                .height_field(ResourceKey::HEIGHT)
                .is_some()
        );
    }

    #[test]
    fn every_generation_starts_with_fresh_resources() {
        let saw_existing = Arc::new(AtomicBool::new(false));
        let execution_count = Arc::new(AtomicUsize::new(0));
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(CheckFreshResourceStage::new(
            ResourceKey::HEIGHT,
            Arc::clone(&saw_existing),
            Arc::clone(&execution_count),
        ));
        let mut runtime = TerrainRuntime::new(layout2(), pipeline);
        let request = request2(
            GenerationSeed::new(1),
            RegionCoord2::ZERO,
            LodLevel::HIGHEST,
        );

        let first = runtime.generate(request).unwrap();
        let second = runtime.generate(request).unwrap();

        assert!(first.resources().contains(ResourceKey::HEIGHT));
        assert!(second.resources().contains(ResourceKey::HEIGHT));
        assert!(!saw_existing.load(Ordering::SeqCst));
        assert_eq!(execution_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn runtime_can_be_reused_for_several_requests() {
        let execution_count = Arc::new(AtomicUsize::new(0));
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(CountingStage::new(Arc::clone(&execution_count)));
        let mut runtime = TerrainRuntime::new(layout2(), pipeline);

        for x in 0..4 {
            runtime
                .generate(request2(
                    GenerationSeed::new(1),
                    RegionCoord2::new(x, 0),
                    LodLevel::HIGHEST,
                ))
                .unwrap();
        }

        assert_eq!(execution_count.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn repeated_deterministic_generation_returns_same_resources() {
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(ProduceHeightFieldStage::new(ResourceKey::HEIGHT, 6.5));
        let mut runtime = TerrainRuntime::new(layout2(), pipeline);
        let request = request2(
            GenerationSeed::new(77),
            RegionCoord2::new(2, 3),
            LodLevel::HIGHEST,
        );

        let first = runtime.generate(request).unwrap();
        let second = runtime.generate(request).unwrap();

        assert_eq!(first.region(), second.region());
        assert_eq!(first.resources(), second.resources());
    }

    #[test]
    fn empty_pipeline_returns_resolved_region_and_empty_resources() {
        let mut runtime = TerrainRuntime::new(layout2(), TerrainPipeline::new());
        let coordinate = RegionCoord2::new(3, -2);

        let result = runtime
            .generate(request2(
                GenerationSeed::new(1),
                coordinate,
                LodLevel::HIGHEST,
            ))
            .unwrap();

        assert_eq!(result.region2().unwrap().coordinate(), coordinate);
        assert!(result.resources().is_empty());
    }

    #[test]
    fn accessors_and_consuming_methods_return_owned_parts() {
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(ProduceHeightFieldStage::new(ResourceKey::HEIGHT, 9.0));
        let mut runtime = TerrainRuntime::new(layout2(), pipeline);
        let request = request2(
            GenerationSeed::new(42),
            RegionCoord2::new(1, 2),
            LodLevel::HIGHEST,
        );
        let result = runtime.generate(request).unwrap();

        assert_eq!(result.request(), request);
        assert_eq!(result.seed(), GenerationSeed::new(42));
        assert_eq!(result.lod(), LodLevel::HIGHEST);
        assert!(result.resources().contains(ResourceKey::HEIGHT));

        let (owned_request, owned_region, owned_resources) = result.into_parts();

        assert_eq!(owned_request, request);
        assert_eq!(
            owned_region.as_region2().unwrap().coordinate(),
            RegionCoord2::new(1, 2)
        );
        assert!(owned_resources.contains(ResourceKey::HEIGHT));
    }

    #[test]
    fn pipeline_accessors_expose_configured_pipeline() {
        let mut runtime = TerrainRuntime::new(layout2(), TerrainPipeline::new());

        assert!(matches!(runtime.layout(), GenerationLayout::Region2(_)));
        assert!(runtime.pipeline().is_empty());

        runtime
            .pipeline_mut()
            .add_stage(CountingStage::new(Arc::new(AtomicUsize::new(0))));

        assert_eq!(runtime.pipeline().stage_count(), 1);
        assert_eq!(runtime.into_pipeline().stage_count(), 1);
    }

    #[test]
    fn conversion_from_pipeline_error_uses_pipeline_execution_variant() {
        let error = RuntimeError::from(PipelineError::new("bad setup"));

        assert!(matches!(error, RuntimeError::PipelineExecution { .. }));
    }

    struct InspectRegion2Stage {
        expected_seed: GenerationSeed,
        expected_coordinate: RegionCoord2,
        expected_lod: LodLevel,
        executed: Arc<AtomicBool>,
    }

    impl InspectRegion2Stage {
        fn new(
            expected_seed: GenerationSeed,
            expected_coordinate: RegionCoord2,
            expected_lod: LodLevel,
            executed: Arc<AtomicBool>,
        ) -> Self {
            Self {
                expected_seed,
                expected_coordinate,
                expected_lod,
                executed,
            }
        }
    }

    impl TerrainStage for InspectRegion2Stage {
        fn id(&self) -> StageId {
            StageId(1)
        }

        fn name(&self) -> &str {
            "inspect_region2"
        }

        fn execute(
            &mut self,
            context: &StageContext,
            _resources: &mut ResourceSet,
        ) -> Result<(), StageError> {
            let region = context.region2()?;

            if context.seed() != self.expected_seed {
                return Err(StageError::new("unexpected seed"));
            }

            if context.lod() != self.expected_lod || region.lod() != self.expected_lod {
                return Err(StageError::new("unexpected lod"));
            }

            if region.coordinate() != self.expected_coordinate {
                return Err(StageError::new("unexpected 2D coordinate"));
            }

            self.executed.store(true, Ordering::SeqCst);

            Ok(())
        }
    }

    struct InspectRegion3Stage {
        expected_seed: GenerationSeed,
        expected_coordinate: RegionCoord3,
        expected_lod: LodLevel,
        executed: Arc<AtomicBool>,
    }

    impl InspectRegion3Stage {
        fn new(
            expected_seed: GenerationSeed,
            expected_coordinate: RegionCoord3,
            expected_lod: LodLevel,
            executed: Arc<AtomicBool>,
        ) -> Self {
            Self {
                expected_seed,
                expected_coordinate,
                expected_lod,
                executed,
            }
        }
    }

    impl TerrainStage for InspectRegion3Stage {
        fn id(&self) -> StageId {
            StageId(1)
        }

        fn name(&self) -> &str {
            "inspect_region3"
        }

        fn execute(
            &mut self,
            context: &StageContext,
            _resources: &mut ResourceSet,
        ) -> Result<(), StageError> {
            let region = context.region3()?;

            if context.seed() != self.expected_seed {
                return Err(StageError::new("unexpected seed"));
            }

            if context.lod() != self.expected_lod || region.lod() != self.expected_lod {
                return Err(StageError::new("unexpected lod"));
            }

            if region.coordinate() != self.expected_coordinate {
                return Err(StageError::new("unexpected 3D coordinate"));
            }

            self.executed.store(true, Ordering::SeqCst);

            Ok(())
        }
    }

    struct ProduceHeightFieldStage {
        output: ResourceKey,
        value: f32,
        products: [ResourceProduct; 1],
    }

    impl ProduceHeightFieldStage {
        fn new(output: ResourceKey, value: f32) -> Self {
            Self {
                output,
                value,
                products: [ResourceProduct::create(output, ResourceKind::HeightField)],
            }
        }
    }

    impl TerrainStage for ProduceHeightFieldStage {
        fn id(&self) -> StageId {
            StageId(2)
        }

        fn name(&self) -> &str {
            "produce_height_field"
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
            let height_field = HeightField::filled(
                region.point_sample_extent(),
                self.value,
                region.transform(),
                SamplingDomain::Points,
                Vector3F64::Y,
            )
            .map_err(StageError::from)?;

            resources.insert(self.output, TerrainResource::HeightField(height_field));

            Ok(())
        }
    }

    struct ProduceDensityFieldStage {
        output: ResourceKey,
        value: f32,
        products: [ResourceProduct; 1],
    }

    impl ProduceDensityFieldStage {
        fn new(output: ResourceKey, value: f32) -> Self {
            Self {
                output,
                value,
                products: [ResourceProduct::create(output, ResourceKind::DensityField)],
            }
        }
    }

    impl TerrainStage for ProduceDensityFieldStage {
        fn id(&self) -> StageId {
            StageId(2)
        }

        fn name(&self) -> &str {
            "produce_density_field"
        }

        fn products(&self) -> &[ResourceProduct] {
            &self.products
        }

        fn execute(
            &mut self,
            context: &StageContext,
            resources: &mut ResourceSet,
        ) -> Result<(), StageError> {
            let region = context.region3()?;
            let density_field = DensityField::filled(
                region.point_sample_extent(),
                self.value,
                region.transform(),
                SamplingDomain::Points,
            )
            .map_err(StageError::from)?;

            resources.insert(self.output, TerrainResource::DensityField(density_field));

            Ok(())
        }
    }

    struct MarkerStage {
        executed: Arc<AtomicBool>,
    }

    impl MarkerStage {
        fn new(executed: Arc<AtomicBool>) -> Self {
            Self { executed }
        }
    }

    impl TerrainStage for MarkerStage {
        fn id(&self) -> StageId {
            StageId(9)
        }

        fn name(&self) -> &str {
            "marker"
        }

        fn execute(
            &mut self,
            _context: &StageContext,
            _resources: &mut ResourceSet,
        ) -> Result<(), StageError> {
            self.executed.store(true, Ordering::SeqCst);

            Ok(())
        }
    }

    struct FailingStage;

    impl TerrainStage for FailingStage {
        fn id(&self) -> StageId {
            StageId(10)
        }

        fn name(&self) -> &str {
            "failing"
        }

        fn execute(
            &mut self,
            _context: &StageContext,
            _resources: &mut ResourceSet,
        ) -> Result<(), StageError> {
            Err(StageError::new("intentional failure"))
        }
    }

    struct CheckFreshResourceStage {
        output: ResourceKey,
        saw_existing: Arc<AtomicBool>,
        execution_count: Arc<AtomicUsize>,
        products: [ResourceProduct; 1],
    }

    impl CheckFreshResourceStage {
        fn new(
            output: ResourceKey,
            saw_existing: Arc<AtomicBool>,
            execution_count: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                output,
                saw_existing,
                execution_count,
                products: [ResourceProduct::create(output, ResourceKind::HeightField)],
            }
        }
    }

    impl TerrainStage for CheckFreshResourceStage {
        fn id(&self) -> StageId {
            StageId(11)
        }

        fn name(&self) -> &str {
            "check_fresh_resource"
        }

        fn products(&self) -> &[ResourceProduct] {
            &self.products
        }

        fn execute(
            &mut self,
            context: &StageContext,
            resources: &mut ResourceSet,
        ) -> Result<(), StageError> {
            if resources.contains(self.output) {
                self.saw_existing.store(true, Ordering::SeqCst);
            }

            self.execution_count.fetch_add(1, Ordering::SeqCst);

            let region = context.region2()?;
            let height_field = HeightField::filled(
                region.point_sample_extent(),
                3.0,
                region.transform(),
                SamplingDomain::Points,
                Vector3F64::Y,
            )
            .map_err(StageError::from)?;

            resources.insert(self.output, TerrainResource::HeightField(height_field));

            Ok(())
        }
    }

    struct CountingStage {
        execution_count: Arc<AtomicUsize>,
    }

    impl CountingStage {
        fn new(execution_count: Arc<AtomicUsize>) -> Self {
            Self { execution_count }
        }
    }

    impl TerrainStage for CountingStage {
        fn id(&self) -> StageId {
            StageId(12)
        }

        fn name(&self) -> &str {
            "counting"
        }

        fn execute(
            &mut self,
            _context: &StageContext,
            _resources: &mut ResourceSet,
        ) -> Result<(), StageError> {
            self.execution_count.fetch_add(1, Ordering::SeqCst);

            Ok(())
        }
    }
}
