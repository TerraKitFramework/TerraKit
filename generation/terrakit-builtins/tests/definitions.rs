use terrakit_builtins::{
    FlatHeightDefinition, HeightFieldMeshDefinition, NoiseHeightDefinition, builtin_stage_registry,
    definition::{AmplifyHeightDefinition, TemperatureMapDefinition},
};
use terrakit_core::{
    Extent2, GenerationSeed, HeightField, LodLevel, RegionCoord2, RegionLayout2, SamplingDomain,
    TerrainMesh, TerrainResource, Vector2F64, Vector3F64,
};
use terrakit_pipeline::{
    ParameterError, ParameterId, ParameterKind, ParameterSet, ParameterType, ParameterValue,
    PipelineAssemblyError, PortId, ResourceAccess, ResourceKey, ResourceKind, ResourceProduct,
    ResourceRequirement, ResourceSet, StageBuildError, StageConstruction, StageDefinition, StageId,
    StageTypeId, TerrainPipelineAssembler, ValidatedStageBindings,
};
use terrakit_runtime::{GenerationRequest, TerrainRuntime};

const BASE_HEIGHT: ResourceKey = ResourceKey(100);
const NOISY_HEIGHT: ResourceKey = ResourceKey(101);
const TERRAIN_MESH: ResourceKey = ResourceKey(102);

fn stage_type(value: &'static str) -> StageTypeId {
    StageTypeId::try_new(value).unwrap()
}

fn port(value: &'static str) -> PortId {
    PortId::try_new(value).unwrap()
}

fn parameter(value: &'static str) -> ParameterId {
    ParameterId::try_new(value).unwrap()
}

fn enum_value(value: &'static str) -> terrakit_pipeline::EnumValueId {
    terrakit_pipeline::EnumValueId::try_new(value).unwrap()
}

fn context() -> terrakit_pipeline::StageContext {
    let layout =
        RegionLayout2::new(Extent2::try_new(3, 2).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
    let region = terrakit_core::GenerationRegion::Region2(
        layout
            .resolve(terrakit_core::RegionRequest2::new(
                RegionCoord2::ZERO,
                LodLevel::HIGHEST,
            ))
            .unwrap(),
    );

    terrakit_pipeline::StageContext::from_u64(123, region)
}

fn source_height_field(value: f32) -> HeightField {
    let context = context();
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

#[test]
fn builtin_registry_contains_all_definitions() {
    let registry = builtin_stage_registry().unwrap();

    assert_eq!(registry.len(), 6);
    assert!(
        registry
            .definition(&stage_type(FlatHeightDefinition::TYPE_ID))
            .is_some()
    );
    assert!(
        registry
            .definition(&stage_type(NoiseHeightDefinition::TYPE_ID))
            .is_some()
    );
    assert!(
        registry
            .definition(&stage_type(HeightFieldMeshDefinition::TYPE_ID))
            .is_some()
    );
    assert!(
        registry
            .definition(&stage_type(TemperatureMapDefinition::TYPE_ID))
            .is_some()
    );
    assert!(
        registry
            .definition(&stage_type(AmplifyHeightDefinition::TYPE_ID))
            .is_some()
    );
}

#[test]
fn builtin_registry_enumerates_schema_versions() {
    let registry = builtin_stage_registry().unwrap();

    assert_eq!(
        registry
            .schemas()
            .map(|schema| (
                schema.type_id().as_str().to_owned(),
                schema.version().value()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("terrakit.height.amplify".to_owned(), 1),
            ("terrakit.height.flat".to_owned(), 1),
            ("terrakit.height.noise".to_owned(), 1),
            ("terrakit.height.temperature".to_owned(), 1),
            ("terrakit.mesh.height_field".to_owned(), 1),
            ("terrakit.scatter.poisson".to_owned(), 1),
        ]
    );
}

#[test]
fn flat_height_schema_snapshot_is_stable() {
    let definition = FlatHeightDefinition::new().unwrap();
    let schema = definition.schema();

    assert_eq!(schema.type_id().as_str(), FlatHeightDefinition::TYPE_ID);
    assert_eq!(schema.version(), FlatHeightDefinition::SCHEMA_VERSION);
    assert!(schema.inputs().is_empty());
    assert_eq!(output_ids(schema), vec!["height"]);
    assert_eq!(
        schema.outputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert_eq!(parameter_ids(schema), vec!["elevation", "height_axis"]);

    let elevation = schema.parameter(&parameter("elevation")).unwrap();
    assert_eq!(
        elevation.value_type(),
        &ParameterType::F32 {
            minimum: None,
            maximum: None,
        }
    );
    assert_eq!(elevation.default(), Some(&ParameterValue::F32(0.0)));

    let height_axis = schema.parameter(&parameter("height_axis")).unwrap();
    assert_eq!(height_axis.value_type(), &ParameterType::Vector3F64);
    assert_eq!(
        height_axis.default(),
        Some(&ParameterValue::Vector3F64(Vector3F64::Y))
    );
}

#[test]
fn noise_height_schema_snapshot_is_stable() {
    let definition = NoiseHeightDefinition::new().unwrap();
    let schema = definition.schema();

    assert_eq!(schema.type_id().as_str(), NoiseHeightDefinition::TYPE_ID);
    assert_eq!(schema.version(), NoiseHeightDefinition::SCHEMA_VERSION);
    assert_eq!(input_ids(schema), vec!["source"]);
    assert_eq!(
        schema.inputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert!(!schema.inputs()[0].optional());
    assert_eq!(output_ids(schema), vec!["height"]);
    assert_eq!(
        schema.outputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert_eq!(
        parameter_ids(schema),
        vec![
            "algorithm",
            "octaves",
            "frequency",
            "lacunarity",
            "persistence",
            "amplitude",
            "normalize",
            "seed_domain",
            "mode",
        ]
    );

    let algorithm = schema.parameter(&parameter("algorithm")).unwrap();
    assert_eq!(
        enum_option_ids(algorithm.value_type()),
        vec!["value", "perlin", "simplex", "worley"]
    );
    assert_eq!(
        algorithm.default(),
        Some(&ParameterValue::Enum(enum_value("value")))
    );

    let octaves = schema.parameter(&parameter("octaves")).unwrap();
    assert_eq!(
        octaves.value_type(),
        &ParameterType::U64 {
            minimum: Some(1),
            maximum: Some(terrakit_algorithms::noise::MAX_FRACTAL_OCTAVES.into()),
        }
    );
    assert_eq!(octaves.default(), Some(&ParameterValue::U64(4)));

    assert_eq!(
        schema
            .parameter(&parameter("frequency"))
            .unwrap()
            .value_type(),
        &ParameterType::F64 {
            minimum: None,
            maximum: None,
        }
    );
    assert_eq!(
        schema.parameter(&parameter("frequency")).unwrap().default(),
        Some(&ParameterValue::F64(0.01))
    );
    assert_eq!(
        schema
            .parameter(&parameter("lacunarity"))
            .unwrap()
            .value_type(),
        &ParameterType::F64 {
            minimum: None,
            maximum: None,
        }
    );
    assert_eq!(
        schema
            .parameter(&parameter("lacunarity"))
            .unwrap()
            .default(),
        Some(&ParameterValue::F64(2.0))
    );
    assert_eq!(
        schema
            .parameter(&parameter("persistence"))
            .unwrap()
            .value_type(),
        &ParameterType::F32 {
            minimum: Some(0.0),
            maximum: None,
        }
    );
    assert_eq!(
        schema
            .parameter(&parameter("persistence"))
            .unwrap()
            .default(),
        Some(&ParameterValue::F32(0.5))
    );
    assert_eq!(
        schema
            .parameter(&parameter("amplitude"))
            .unwrap()
            .value_type(),
        &ParameterType::F32 {
            minimum: None,
            maximum: None,
        }
    );
    assert_eq!(
        schema.parameter(&parameter("amplitude")).unwrap().default(),
        Some(&ParameterValue::F32(1.0))
    );
    assert_eq!(
        schema
            .parameter(&parameter("normalize"))
            .unwrap()
            .value_type(),
        &ParameterType::Bool
    );
    assert_eq!(
        schema.parameter(&parameter("normalize")).unwrap().default(),
        Some(&ParameterValue::Bool(true))
    );
    assert_eq!(
        schema
            .parameter(&parameter("seed_domain"))
            .unwrap()
            .value_type(),
        &ParameterType::U64 {
            minimum: None,
            maximum: None,
        }
    );
    assert_eq!(
        schema
            .parameter(&parameter("seed_domain"))
            .unwrap()
            .default(),
        Some(&ParameterValue::U64(0))
    );

    let mode = schema.parameter(&parameter("mode")).unwrap();
    assert_eq!(
        enum_option_ids(mode.value_type()),
        vec!["add", "replace", "multiply"]
    );
    assert_eq!(
        mode.default(),
        Some(&ParameterValue::Enum(enum_value("add")))
    );
}

#[test]
fn heightfield_mesh_schema_snapshot_is_stable() {
    let definition = HeightFieldMeshDefinition::new().unwrap();
    let schema = definition.schema();

    assert_eq!(
        schema.type_id().as_str(),
        HeightFieldMeshDefinition::TYPE_ID
    );
    assert_eq!(schema.version(), HeightFieldMeshDefinition::SCHEMA_VERSION);
    assert_eq!(input_ids(schema), vec!["height"]);
    assert_eq!(
        schema.inputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert!(!schema.inputs()[0].optional());
    assert_eq!(output_ids(schema), vec!["mesh"]);
    assert_eq!(schema.outputs()[0].resource_kind(), ResourceKind::Mesh);
    assert_eq!(
        parameter_ids(schema),
        vec!["generate_normals", "generate_texcoords"]
    );
    assert_eq!(
        schema
            .parameter(&parameter("generate_normals"))
            .unwrap()
            .value_type(),
        &ParameterType::Bool
    );
    assert_eq!(
        schema
            .parameter(&parameter("generate_normals"))
            .unwrap()
            .default(),
        Some(&ParameterValue::Bool(true))
    );
    assert_eq!(
        schema
            .parameter(&parameter("generate_texcoords"))
            .unwrap()
            .value_type(),
        &ParameterType::Bool
    );
    assert_eq!(
        schema
            .parameter(&parameter("generate_texcoords"))
            .unwrap()
            .default(),
        Some(&ParameterValue::Bool(true))
    );
}

#[test]
fn flat_height_definition_schema_and_factory_are_valid() {
    let definition = FlatHeightDefinition::new().unwrap();
    let schema = definition.schema();

    assert_eq!(schema.type_id().as_str(), "terrakit.height.flat");
    assert_eq!(schema.display_name(), "Flat Height");
    assert_eq!(schema.category(), "Height");
    assert!(schema.inputs().is_empty());
    assert_eq!(schema.outputs()[0].id().as_str(), "height");
    assert_eq!(
        schema.outputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert_eq!(schema.parameters()[0].id().as_str(), "elevation");
    assert_eq!(schema.parameters()[1].id().as_str(), "height_axis");

    let mut supplied = ParameterSet::new();
    supplied.set(parameter("elevation"), ParameterValue::F32(4.5));
    let parameters = schema.resolve_parameters(&supplied).unwrap();
    let bindings = ValidatedStageBindings::validate(
        schema,
        &terrakit_pipeline::StageBindings::new()
            .with_output(port("height"), BASE_HEIGHT)
            .unwrap(),
    )
    .unwrap();
    let mut stage = definition
        .build(StageId(1), &parameters, &bindings)
        .unwrap();

    assert_eq!(stage.name(), "flat_height");
    assert_eq!(
        stage.products(),
        &[ResourceProduct::create(
            BASE_HEIGHT,
            ResourceKind::HeightField,
        )]
    );

    let mut resources = ResourceSet::new();
    stage.execute(&context(), &mut resources).unwrap();
    assert!(
        resources
            .height_field(BASE_HEIGHT)
            .unwrap()
            .values()
            .iter()
            .all(|value| *value == 4.5)
    );
}

#[test]
fn flat_height_domain_validation_returns_stage_build_error() {
    let definition = FlatHeightDefinition::new().unwrap();
    let schema = definition.schema();
    let mut supplied = ParameterSet::new();
    supplied.set(
        parameter("height_axis"),
        ParameterValue::Vector3F64(Vector3F64::ZERO),
    );
    let parameters = schema.resolve_parameters(&supplied).unwrap();
    let bindings = ValidatedStageBindings::validate(
        schema,
        &terrakit_pipeline::StageBindings::new()
            .with_output(port("height"), BASE_HEIGHT)
            .unwrap(),
    )
    .unwrap();

    assert!(matches!(
        definition.build(StageId(1), &parameters, &bindings),
        Err(StageBuildError::Stage { .. })
    ));
}

#[test]
fn noise_height_definition_schema_and_default_factory_are_valid() {
    let definition = NoiseHeightDefinition::new().unwrap();
    let schema = definition.schema();

    assert_eq!(schema.type_id().as_str(), "terrakit.height.noise");
    assert_eq!(schema.display_name(), "Noise Height");
    assert_eq!(schema.category(), "Height");
    assert_eq!(schema.inputs()[0].id().as_str(), "source");
    assert_eq!(
        schema.inputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert_eq!(schema.outputs()[0].id().as_str(), "height");
    assert_eq!(
        schema.outputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert!(schema.parameter(&parameter("algorithm")).is_some());
    assert!(schema.parameter(&parameter("octaves")).is_some());
    assert!(schema.parameter(&parameter("mode")).is_some());

    let parameters = schema.resolve_parameters(&ParameterSet::new()).unwrap();
    assert_eq!(parameters.enum_id("algorithm").unwrap().as_str(), "value");
    assert_eq!(parameters.u64("octaves").unwrap(), 4);
    assert_eq!(parameters.enum_id("mode").unwrap().as_str(), "add");

    let bindings = ValidatedStageBindings::validate(
        schema,
        &terrakit_pipeline::StageBindings::new()
            .with_input(port("source"), BASE_HEIGHT)
            .unwrap()
            .with_output(port("height"), NOISY_HEIGHT)
            .unwrap(),
    )
    .unwrap();
    let stage = definition
        .build(StageId(2), &parameters, &bindings)
        .unwrap();

    assert_eq!(stage.name(), "noise_height");
    assert_eq!(
        stage.requirements(),
        &[ResourceRequirement::required(
            BASE_HEIGHT,
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
fn noise_height_definition_builds_all_algorithm_options() {
    for algorithm in ["value", "perlin", "simplex", "worley"] {
        let definition = NoiseHeightDefinition::new().unwrap();
        let schema = definition.schema();
        let mut supplied = ParameterSet::new();
        supplied.set(
            parameter("algorithm"),
            ParameterValue::Enum(enum_value(algorithm)),
        );
        supplied.set(parameter("frequency"), ParameterValue::F64(0.2));
        supplied.set(parameter("amplitude"), ParameterValue::F32(0.5));
        let parameters = schema.resolve_parameters(&supplied).unwrap();
        let bindings = ValidatedStageBindings::validate(
            schema,
            &terrakit_pipeline::StageBindings::new()
                .with_input(port("source"), BASE_HEIGHT)
                .unwrap()
                .with_output(port("height"), NOISY_HEIGHT)
                .unwrap(),
        )
        .unwrap();
        let mut stage = definition
            .build(StageId(2), &parameters, &bindings)
            .unwrap();
        let mut resources = ResourceSet::new();
        resources.insert(
            BASE_HEIGHT,
            TerrainResource::HeightField(source_height_field(0.0)),
        );

        stage.execute(&context(), &mut resources).unwrap();

        let output = resources.height_field(NOISY_HEIGHT).unwrap();
        assert!(
            output.values().iter().all(|value| value.is_finite()),
            "{algorithm} produced non-finite height values"
        );
        assert!(
            output.values().iter().any(|value| *value != 0.0),
            "{algorithm} did not affect the height field"
        );
    }
}

#[test]
fn noise_height_definition_respects_modes_and_preserves_source() {
    for (mode, source_value) in [("add", 2.0), ("replace", 2.0), ("multiply", 2.0)] {
        let definition = NoiseHeightDefinition::new().unwrap();
        let schema = definition.schema();
        let mut supplied = ParameterSet::new();
        supplied.set(
            parameter("mode"),
            ParameterValue::Enum(terrakit_pipeline::EnumValueId::try_new(mode).unwrap()),
        );
        supplied.set(parameter("amplitude"), ParameterValue::F32(0.0));
        let parameters = schema.resolve_parameters(&supplied).unwrap();
        let bindings = ValidatedStageBindings::validate(
            schema,
            &terrakit_pipeline::StageBindings::new()
                .with_input(port("source"), BASE_HEIGHT)
                .unwrap()
                .with_output(port("height"), NOISY_HEIGHT)
                .unwrap(),
        )
        .unwrap();
        let mut stage = definition
            .build(StageId(2), &parameters, &bindings)
            .unwrap();
        let original = source_height_field(source_value);
        let mut resources = ResourceSet::new();
        resources.insert(BASE_HEIGHT, TerrainResource::HeightField(original.clone()));

        stage.execute(&context(), &mut resources).unwrap();

        assert_eq!(resources.height_field(BASE_HEIGHT), Some(&original));
        let output = resources.height_field(NOISY_HEIGHT).unwrap();
        match mode {
            "add" => assert!(output.values().iter().all(|value| *value == source_value)),
            "replace" => assert!(output.values().iter().all(|value| *value == 0.0)),
            "multiply" => assert!(output.values().iter().all(|value| *value == 0.0)),
            _ => unreachable!(),
        }
    }
}

#[test]
fn noise_height_wrong_types_and_domain_errors_are_reported() {
    let definition = NoiseHeightDefinition::new().unwrap();
    let schema = definition.schema();
    let mut wrong_type = ParameterSet::new();
    wrong_type.set(parameter("octaves"), ParameterValue::F32(4.0));
    assert!(matches!(
        schema.resolve_parameters(&wrong_type),
        Err(ParameterError::WrongValueType {
            expected: ParameterKind::U64,
            actual: ParameterKind::F32,
            ..
        })
    ));

    let mut invalid_domain = ParameterSet::new();
    invalid_domain.set(parameter("frequency"), ParameterValue::F64(0.0));
    let parameters = schema.resolve_parameters(&invalid_domain).unwrap();
    let bindings = ValidatedStageBindings::validate(
        schema,
        &terrakit_pipeline::StageBindings::new()
            .with_input(port("source"), BASE_HEIGHT)
            .unwrap()
            .with_output(port("height"), NOISY_HEIGHT)
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        definition.build(StageId(2), &parameters, &bindings),
        Err(StageBuildError::Stage { .. })
    ));
}

#[test]
fn heightfield_mesh_definition_schema_and_factory_are_valid() {
    let definition = HeightFieldMeshDefinition::new().unwrap();
    let schema = definition.schema();

    assert_eq!(schema.type_id().as_str(), "terrakit.mesh.height_field");
    assert_eq!(schema.display_name(), "Heightfield Mesh");
    assert_eq!(schema.category(), "Mesh");
    assert_eq!(schema.inputs()[0].id().as_str(), "height");
    assert_eq!(
        schema.inputs()[0].resource_kind(),
        ResourceKind::HeightField
    );
    assert_eq!(schema.outputs()[0].id().as_str(), "mesh");
    assert_eq!(schema.outputs()[0].resource_kind(), ResourceKind::Mesh);

    let mut supplied = ParameterSet::new();
    supplied.set(parameter("generate_normals"), ParameterValue::Bool(false));
    supplied.set(parameter("generate_texcoords"), ParameterValue::Bool(false));
    let parameters = schema.resolve_parameters(&supplied).unwrap();
    let bindings = ValidatedStageBindings::validate(
        schema,
        &terrakit_pipeline::StageBindings::new()
            .with_input(port("height"), NOISY_HEIGHT)
            .unwrap()
            .with_output(port("mesh"), TERRAIN_MESH)
            .unwrap(),
    )
    .unwrap();
    let mut stage = definition
        .build(StageId(3), &parameters, &bindings)
        .unwrap();

    assert_eq!(stage.name(), "height_field_mesh");
    assert_eq!(
        stage.products(),
        &[ResourceProduct::create(TERRAIN_MESH, ResourceKind::Mesh)]
    );

    let mut resources = ResourceSet::new();
    resources.insert(
        NOISY_HEIGHT,
        TerrainResource::HeightField(source_height_field(1.0)),
    );
    stage.execute(&context(), &mut resources).unwrap();
    let mesh = resources.mesh(TERRAIN_MESH).unwrap();
    assert!(mesh.normals().is_none());
    assert!(mesh.texcoords().is_none());
}

#[test]
fn complete_pipeline_can_be_assembled_and_run_through_runtime() {
    let mut runtime = assembled_runtime();
    let request = GenerationRequest::region2(
        GenerationSeed::new(55),
        RegionCoord2::ZERO,
        LodLevel::HIGHEST,
    );

    let first = runtime.generate(request).unwrap();
    let second = runtime.generate(request).unwrap();

    assert_eq!(first.resources(), second.resources());
    assert!(first.resources().height_field(BASE_HEIGHT).is_some());
    assert!(first.resources().height_field(NOISY_HEIGHT).is_some());
    assert!(first.resources().mesh(TERRAIN_MESH).is_some());
    assert!(
        first
            .resources()
            .height_field(BASE_HEIGHT)
            .unwrap()
            .values()
            .iter()
            .all(|value| *value == 0.0)
    );
    assert!(
        first
            .resources()
            .height_field(NOISY_HEIGHT)
            .unwrap()
            .values()
            .iter()
            .any(|value| *value != 0.0)
    );
    assert_mesh_matches_heightfield(
        first.resources().height_field(NOISY_HEIGHT).unwrap(),
        first.resources().mesh(TERRAIN_MESH).unwrap(),
    );
}

#[test]
fn assembled_runtime_preserves_adjacent_region_continuity() {
    let mut runtime = assembled_runtime();
    let left = runtime
        .generate(GenerationRequest::region2(
            GenerationSeed::new(55),
            RegionCoord2::ZERO,
            LodLevel::HIGHEST,
        ))
        .unwrap();
    let right = runtime
        .generate(GenerationRequest::region2(
            GenerationSeed::new(55),
            RegionCoord2::new(1, 0),
            LodLevel::HIGHEST,
        ))
        .unwrap();
    let left_height = left.resources().height_field(NOISY_HEIGHT).unwrap();
    let right_height = right.resources().height_field(NOISY_HEIGHT).unwrap();
    let edge_x = left_height.width() - 1;

    for y in 0..left_height.height() {
        assert_eq!(left_height.get(edge_x, y), right_height.get(0, y));
    }
}

#[test]
fn assembler_rejects_built_in_contract_violations_before_runtime() {
    let registry = builtin_stage_registry().unwrap();
    let mut assembler = TerrainPipelineAssembler::new(&registry);
    let error = assembler
        .add_stage(
            StageConstruction::new(
                StageId(1),
                stage_type(NoiseHeightDefinition::TYPE_ID),
                NoiseHeightDefinition::SCHEMA_VERSION,
            )
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_input(port("source"), BASE_HEIGHT)
                    .unwrap()
                    .with_output(port("height"), BASE_HEIGHT)
                    .unwrap(),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        PipelineAssemblyError::InputOutputAlias {
            key: BASE_HEIGHT,
            ..
        }
    ));
}

fn assembled_runtime() -> TerrainRuntime {
    let layout =
        RegionLayout2::new(Extent2::try_new(8, 6).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
    let registry = builtin_stage_registry().unwrap();
    let mut assembler = TerrainPipelineAssembler::new(&registry);

    let mut flat_parameters = ParameterSet::new();
    flat_parameters.set(parameter("elevation"), ParameterValue::F32(0.0));
    assembler
        .add_stage(
            StageConstruction::new(
                StageId(1),
                stage_type(FlatHeightDefinition::TYPE_ID),
                FlatHeightDefinition::SCHEMA_VERSION,
            )
            .with_parameters(flat_parameters)
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_output(port("height"), BASE_HEIGHT)
                    .unwrap(),
            ),
        )
        .unwrap();

    let mut noise_parameters = ParameterSet::new();
    noise_parameters.set(parameter("frequency"), ParameterValue::F64(0.21));
    noise_parameters.set(parameter("amplitude"), ParameterValue::F32(0.8));
    noise_parameters.set(parameter("seed_domain"), ParameterValue::U64(9));
    assembler
        .add_stage(
            StageConstruction::new(
                StageId(2),
                stage_type(NoiseHeightDefinition::TYPE_ID),
                NoiseHeightDefinition::SCHEMA_VERSION,
            )
            .with_parameters(noise_parameters)
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_input(port("source"), BASE_HEIGHT)
                    .unwrap()
                    .with_output(port("height"), NOISY_HEIGHT)
                    .unwrap(),
            ),
        )
        .unwrap();

    assembler
        .add_stage(
            StageConstruction::new(
                StageId(3),
                stage_type(HeightFieldMeshDefinition::TYPE_ID),
                HeightFieldMeshDefinition::SCHEMA_VERSION,
            )
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_input(port("height"), NOISY_HEIGHT)
                    .unwrap()
                    .with_output(port("mesh"), TERRAIN_MESH)
                    .unwrap(),
            ),
        )
        .unwrap();

    TerrainRuntime::new(layout, assembler.finish())
}

fn assert_mesh_matches_heightfield(height_field: &HeightField, mesh: &TerrainMesh) {
    assert_eq!(mesh.vertex_count(), height_field.values().len());

    for y in 0..height_field.height() {
        for x in 0..height_field.width() {
            let index = y * height_field.width() + x;
            let world = height_field.surface_position(x, y).unwrap();
            let local = mesh.positions()[index];
            let reconstructed = Vector3F64::new(
                mesh.origin().x + f64::from(local.x),
                mesh.origin().y + f64::from(local.y),
                mesh.origin().z + f64::from(local.z),
            );

            assert_eq!(world, reconstructed);
        }
    }
}

fn input_ids(schema: &terrakit_pipeline::StageSchema) -> Vec<&str> {
    schema
        .inputs()
        .iter()
        .map(|input| input.id().as_str())
        .collect()
}

fn output_ids(schema: &terrakit_pipeline::StageSchema) -> Vec<&str> {
    schema
        .outputs()
        .iter()
        .map(|output| output.id().as_str())
        .collect()
}

fn parameter_ids(schema: &terrakit_pipeline::StageSchema) -> Vec<&str> {
    schema
        .parameters()
        .iter()
        .map(|parameter| parameter.id().as_str())
        .collect()
}

fn enum_option_ids(value_type: &ParameterType) -> Vec<&str> {
    let ParameterType::Enum { options } = value_type else {
        panic!("expected enum parameter type");
    };

    options.iter().map(|option| option.id().as_str()).collect()
}
