use std::{
    error::Error,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use terrakit_core::{
    Extent2, Extent3, GenerationRegion, GridTransform2, HeightField, LodLevel, RegionCoord2,
    RegionCoord3, RegionLayout2, RegionLayout3, RegionRequest2, RegionRequest3, SamplingDomain,
    TerrainMesh, TerrainResource, Vector2F64, Vector3F32, Vector3F64,
};
use terrakit_pipeline::{
    PipelineError, ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement,
    ResourceSet, StageContext, StageError, StageId, TerrainPipeline, TerrainStage,
};

fn height_field(value: f32) -> HeightField {
    HeightField::filled(
        Extent2::try_new(4, 3).unwrap(),
        value,
        GridTransform2::identity_xz(),
        SamplingDomain::Points,
        Vector3F64::Y,
    )
    .unwrap()
}

fn region() -> GenerationRegion {
    let layout =
        RegionLayout2::new(Extent2::try_new(4, 3).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();

    GenerationRegion::Region2(
        layout
            .resolve(RegionRequest2::new(
                RegionCoord2::new(2, -1),
                LodLevel::HIGHEST,
            ))
            .unwrap(),
    )
}

fn region2_with_lod(lod: LodLevel) -> GenerationRegion {
    let layout =
        RegionLayout2::new(Extent2::try_new(8, 4).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();

    GenerationRegion::Region2(
        layout
            .resolve(RegionRequest2::new(RegionCoord2::new(2, -1), lod))
            .unwrap(),
    )
}

fn region3_with_lod(lod: LodLevel) -> GenerationRegion {
    let layout = RegionLayout3::new(
        Extent3::try_new(8, 4, 4).unwrap(),
        Vector3F64::new(1.0, 1.0, 1.0),
    )
    .unwrap();

    GenerationRegion::Region3(
        layout
            .resolve(RegionRequest3::new(RegionCoord3::new(1, 2, -3), lod))
            .unwrap(),
    )
}

fn context(seed: u64) -> StageContext {
    StageContext::from_u64(seed, region())
}

fn mesh() -> TerrainMesh {
    TerrainMesh::new(
        Vector3F64::ZERO,
        vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(0.0, 0.0, 1.0),
        ],
        vec![0, 1, 2],
        None,
        None,
    )
    .unwrap()
}

struct CreateHeightStage {
    output: ResourceKey,
    products: [ResourceProduct; 1],
}

impl CreateHeightStage {
    fn new(output: ResourceKey) -> Self {
        Self {
            output,
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
        }
    }
}

impl TerrainStage for CreateHeightStage {
    fn id(&self) -> StageId {
        StageId(40)
    }

    fn name(&self) -> &str {
        "create_height"
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        _context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        resources.insert(
            self.output,
            TerrainResource::HeightField(height_field(10.0)),
        );

        Ok(())
    }
}

struct AddNoiseStage {
    target: ResourceKey,
    amplitude: f32,
    domain: u64,
    requirements: [ResourceRequirement; 1],
}

impl AddNoiseStage {
    fn new(target: ResourceKey, amplitude: f32, domain: u64) -> Self {
        Self {
            target,
            amplitude,
            domain,
            requirements: [ResourceRequirement::required(
                target,
                ResourceKind::HeightField,
                ResourceAccess::ReadWrite,
            )],
        }
    }
}

impl TerrainStage for AddNoiseStage {
    fn id(&self) -> StageId {
        StageId(41)
    }

    fn name(&self) -> &str {
        "add_noise"
    }

    fn requirements(&self) -> &[ResourceRequirement] {
        &self.requirements
    }

    fn execute(
        &mut self,
        context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        let seed = context.derived_seed(self.domain);
        let Some(height_field) = resources.height_field_mut(self.target) else {
            return Err(StageError::new("target height field is missing"));
        };

        for y in 0..height_field.height() {
            for x in 0..height_field.width() {
                let Some(current) = height_field.get(x, y) else {
                    return Err(StageError::new("height sample is missing"));
                };

                let next = current + coordinate_noise(x, y, seed) * self.amplitude;
                height_field.try_set(x, y, next).map_err(StageError::from)?;
            }
        }

        Ok(())
    }
}

fn run_height_pipeline(seed: u64) -> ResourceSet {
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(CreateHeightStage::new(ResourceKey::HEIGHT));
    pipeline.add_stage(AddNoiseStage::new(ResourceKey::HEIGHT, 0.75, 99));

    let mut resources = ResourceSet::new();
    pipeline.execute(&context(seed), &mut resources).unwrap();

    resources
}

fn coordinate_noise(x: usize, y: usize, seed: u64) -> f32 {
    let mut value = seed;

    value ^= (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value ^= (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = mix_u64(value);

    let normalised = value as f64 / u64::MAX as f64;
    (normalised as f32 * 2.0) - 1.0
}

fn mix_u64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

struct MarkerStage {
    id: StageId,
    marker: u64,
    log: Arc<Mutex<Vec<u64>>>,
}

impl TerrainStage for MarkerStage {
    fn id(&self) -> StageId {
        self.id
    }

    fn name(&self) -> &str {
        "marker"
    }

    fn execute(
        &mut self,
        _context: &StageContext,
        _resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        self.log.lock().unwrap().push(self.marker);

        Ok(())
    }
}

struct InspectRegionStage {
    seen: Arc<Mutex<Option<RegionCoord2>>>,
}

impl TerrainStage for InspectRegionStage {
    fn id(&self) -> StageId {
        StageId(42)
    }

    fn name(&self) -> &str {
        "inspect_region"
    }

    fn execute(
        &mut self,
        context: &StageContext,
        _resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        *self.seen.lock().unwrap() = Some(context.region2()?.coordinate());

        Ok(())
    }
}

struct AssertLodStage {
    expected: LodLevel,
    seen: Arc<Mutex<Option<LodLevel>>>,
}

impl TerrainStage for AssertLodStage {
    fn id(&self) -> StageId {
        StageId(43)
    }

    fn name(&self) -> &str {
        "assert_lod"
    }

    fn execute(
        &mut self,
        context: &StageContext,
        _resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        let actual = context.lod();
        *self.seen.lock().unwrap() = Some(actual);

        if actual != self.expected {
            return Err(StageError::new(format!(
                "expected detail level {}, received {}",
                self.expected.value(),
                actual.value()
            )));
        }

        Ok(())
    }
}

struct RequiredHeightStage {
    id: StageId,
    name: &'static str,
    executed: Arc<AtomicBool>,
    requirements: [ResourceRequirement; 1],
}

impl RequiredHeightStage {
    fn new(id: StageId, executed: Arc<AtomicBool>) -> Self {
        Self {
            id,
            name: "required_height",
            executed,
            requirements: [ResourceRequirement::required(
                ResourceKey::HEIGHT,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],
        }
    }
}

impl TerrainStage for RequiredHeightStage {
    fn id(&self) -> StageId {
        self.id
    }

    fn name(&self) -> &str {
        self.name
    }

    fn requirements(&self) -> &[ResourceRequirement] {
        &self.requirements
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

enum ProductBehavior {
    None,
    Mesh,
}

struct ProductStage {
    id: StageId,
    name: &'static str,
    executed: Arc<AtomicBool>,
    products: [ResourceProduct; 1],
    behavior: ProductBehavior,
}

impl ProductStage {
    fn new(executed: Arc<AtomicBool>, behavior: ProductBehavior) -> Self {
        Self {
            id: StageId(20),
            name: "product",
            executed,
            products: [ResourceProduct::create(
                ResourceKey::HEIGHT,
                ResourceKind::HeightField,
            )],
            behavior,
        }
    }
}

impl TerrainStage for ProductStage {
    fn id(&self) -> StageId {
        self.id
    }

    fn name(&self) -> &str {
        self.name
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        _context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        self.executed.store(true, Ordering::SeqCst);

        match self.behavior {
            ProductBehavior::None => {}
            ProductBehavior::Mesh => {
                resources.insert(ResourceKey::HEIGHT, TerrainResource::Mesh(mesh()));
            }
        }

        Ok(())
    }
}

struct FailureStage {
    id: StageId,
}

impl TerrainStage for FailureStage {
    fn id(&self) -> StageId {
        self.id
    }

    fn name(&self) -> &str {
        "failure"
    }

    fn execute(
        &mut self,
        _context: &StageContext,
        _resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        Err(StageError::new("known failure"))
    }
}

#[test]
fn stages_execute_in_insertion_order() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut pipeline = TerrainPipeline::new();

    pipeline.add_stage(MarkerStage {
        id: StageId(1),
        marker: 1,
        log: Arc::clone(&log),
    });
    pipeline.add_stage(MarkerStage {
        id: StageId(2),
        marker: 2,
        log: Arc::clone(&log),
    });

    pipeline
        .execute(&context(0), &mut ResourceSet::new())
        .unwrap();

    assert_eq!(*log.lock().unwrap(), vec![1, 2]);
}

#[test]
fn test_only_stages_can_inspect_region_information() {
    let seen = Arc::new(Mutex::new(None));
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(InspectRegionStage {
        seen: Arc::clone(&seen),
    });

    pipeline
        .execute(&context(0), &mut ResourceSet::new())
        .unwrap();

    assert_eq!(*seen.lock().unwrap(), Some(RegionCoord2::new(2, -1)));
}

#[test]
fn test_only_stages_receive_requested_lod_for_2d_regions() {
    let expected = LodLevel::new(2);
    let seen = Arc::new(Mutex::new(None));
    let context = StageContext::from_u64(0, region2_with_lod(expected));
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(AssertLodStage {
        expected,
        seen: Arc::clone(&seen),
    });

    pipeline.execute(&context, &mut ResourceSet::new()).unwrap();

    assert_eq!(*seen.lock().unwrap(), Some(expected));
}

#[test]
fn test_only_stages_receive_requested_lod_for_3d_regions() {
    let expected = LodLevel::new(1);
    let seen = Arc::new(Mutex::new(None));
    let context = StageContext::from_u64(0, region3_with_lod(expected));
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(AssertLodStage {
        expected,
        seen: Arc::clone(&seen),
    });

    pipeline.execute(&context, &mut ResourceSet::new()).unwrap();

    assert_eq!(*seen.lock().unwrap(), Some(expected));
}

#[test]
fn test_only_lod_stage_fails_when_context_level_differs() {
    let seen = Arc::new(Mutex::new(None));
    let context = StageContext::from_u64(0, region2_with_lod(LodLevel::new(1)));
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(AssertLodStage {
        expected: LodLevel::new(2),
        seen: Arc::clone(&seen),
    });

    let error = pipeline
        .execute(&context, &mut ResourceSet::new())
        .unwrap_err();

    match error {
        PipelineError::StageFailed { source, .. } => {
            assert!(source.message().contains("expected detail level 2"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert_eq!(*seen.lock().unwrap(), Some(LodLevel::new(1)));
}

#[test]
fn missing_requirement_fails_before_stage_execution() {
    let executed = Arc::new(AtomicBool::new(false));
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(RequiredHeightStage::new(StageId(10), Arc::clone(&executed)));

    let error = pipeline
        .execute(&context(0), &mut ResourceSet::new())
        .unwrap_err();

    match error {
        PipelineError::MissingResource {
            stage_id,
            stage_name,
            key,
        } => {
            assert_eq!(stage_id, StageId(10));
            assert_eq!(stage_name, "required_height");
            assert_eq!(key, ResourceKey::HEIGHT);
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert!(!executed.load(Ordering::SeqCst));
}

#[test]
fn wrong_input_kind_is_rejected() {
    let executed = Arc::new(AtomicBool::new(false));
    let mut resources = ResourceSet::new();
    resources.insert(ResourceKey::HEIGHT, TerrainResource::Mesh(mesh()));

    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(RequiredHeightStage::new(StageId(11), Arc::clone(&executed)));

    let error = pipeline.execute(&context(0), &mut resources).unwrap_err();

    match error {
        PipelineError::WrongResourceKind {
            stage_id,
            key,
            expected,
            actual,
            ..
        } => {
            assert_eq!(stage_id, StageId(11));
            assert_eq!(key, ResourceKey::HEIGHT);
            assert_eq!(expected, ResourceKind::HeightField);
            assert_eq!(actual, ResourceKind::Mesh);
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert!(!executed.load(Ordering::SeqCst));
}

#[test]
fn product_collision_fails_before_stage_execution() {
    let executed = Arc::new(AtomicBool::new(false));
    let mut resources = ResourceSet::new();
    resources.insert(
        ResourceKey::HEIGHT,
        TerrainResource::HeightField(height_field(1.0)),
    );

    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(ProductStage::new(
        Arc::clone(&executed),
        ProductBehavior::None,
    ));

    let error = pipeline.execute(&context(0), &mut resources).unwrap_err();

    match error {
        PipelineError::ResourceAlreadyExists { key, .. } => {
            assert_eq!(key, ResourceKey::HEIGHT);
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert!(!executed.load(Ordering::SeqCst));
}

#[test]
fn missing_output_is_rejected_after_stage_execution() {
    let executed = Arc::new(AtomicBool::new(false));
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(ProductStage::new(
        Arc::clone(&executed),
        ProductBehavior::None,
    ));

    let error = pipeline
        .execute(&context(0), &mut ResourceSet::new())
        .unwrap_err();

    match error {
        PipelineError::MissingOutput { key, .. } => {
            assert_eq!(key, ResourceKey::HEIGHT);
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert!(executed.load(Ordering::SeqCst));
}

#[test]
fn wrong_output_kind_is_rejected_after_stage_execution() {
    let executed = Arc::new(AtomicBool::new(false));
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(ProductStage::new(
        Arc::clone(&executed),
        ProductBehavior::Mesh,
    ));

    let error = pipeline
        .execute(&context(0), &mut ResourceSet::new())
        .unwrap_err();

    match error {
        PipelineError::WrongOutputKind {
            key,
            expected,
            actual,
            ..
        } => {
            assert_eq!(key, ResourceKey::HEIGHT);
            assert_eq!(expected, ResourceKind::HeightField);
            assert_eq!(actual, ResourceKind::Mesh);
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert!(executed.load(Ordering::SeqCst));
}

#[test]
fn stage_failure_preserves_context_and_source() {
    let mut pipeline = TerrainPipeline::new();
    pipeline.add_stage(FailureStage { id: StageId(30) });

    let error = pipeline
        .execute(&context(0), &mut ResourceSet::new())
        .unwrap_err();

    match &error {
        PipelineError::StageFailed {
            stage_id,
            stage_name,
            source,
        } => {
            assert_eq!(*stage_id, StageId(30));
            assert_eq!(stage_name, "failure");
            assert_eq!(source.message(), "known failure");
        }
        other => panic!("unexpected error: {other:?}"),
    }

    assert_eq!(
        error.source().map(ToString::to_string),
        Some("known failure".to_owned())
    );
}

#[test]
fn same_seed_produces_identical_height_values() {
    let first = run_height_pipeline(123);
    let second = run_height_pipeline(123);

    assert_eq!(
        first.height_field(ResourceKey::HEIGHT).unwrap().values(),
        second.height_field(ResourceKey::HEIGHT).unwrap().values()
    );
}

#[test]
fn different_seeds_change_at_least_one_height_value() {
    let first = run_height_pipeline(123);
    let second = run_height_pipeline(124);

    let first_values = first.height_field(ResourceKey::HEIGHT).unwrap().values();
    let second_values = second.height_field(ResourceKey::HEIGHT).unwrap().values();

    assert!(
        first_values
            .iter()
            .zip(second_values.iter())
            .any(|(first, second)| first != second)
    );
}

#[test]
fn generated_height_field_remains_valid_according_to_core() {
    let resources = run_height_pipeline(123);
    let height_field = resources.height_field(ResourceKey::HEIGHT).unwrap();

    let rebuilt = HeightField::from_values(
        height_field.extent(),
        height_field.values().to_vec(),
        height_field.transform(),
        height_field.sampling(),
        height_field.height_axis(),
    );

    assert!(rebuilt.is_ok());
}
