use std::{
    collections::{BTreeSet, HashSet},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use terrakit_core::{
    Extent2, GenerationRegion, GenerationSeed, GridTransform2, HeightField, LodLevel, RegionCoord2,
    RegionLayout2, RegionRequest2, SamplingDomain, TerrainMesh, TerrainResource, Vector2F64,
    Vector3F32, Vector3F64,
};
use terrakit_pipeline::{
    BindingError, DefinitionError, EnumOption, EnumValueId, InputPortDefinition,
    OutputPortDefinition, ParameterDefinition, ParameterError, ParameterId, ParameterKind,
    ParameterSet, ParameterType, ParameterValue, PipelineAssemblyError, PortId,
    ResolvedParameterSet, ResourceAccess, ResourceKey, ResourceKind, ResourceProduct,
    ResourceRequirement, ResourceSet, StageBuildError, StageConstruction, StageDefinition, StageId,
    StageRegistry, StageSchema, StageSchemaVersion, StageTypeId, TerrainPipelineAssembler,
    TerrainStage, ValidatedStageBindings,
};

fn stage_type(value: &'static str) -> StageTypeId {
    StageTypeId::try_new(value).unwrap()
}

fn port(value: &'static str) -> PortId {
    PortId::try_new(value).unwrap()
}

fn parameter(value: &'static str) -> ParameterId {
    ParameterId::try_new(value).unwrap()
}

fn enum_value(value: &'static str) -> EnumValueId {
    EnumValueId::try_new(value).unwrap()
}

fn enum_option(value: &'static str) -> EnumOption {
    EnumOption::new(enum_value(value), value, "").unwrap()
}

fn input(id: &'static str, kind: ResourceKind) -> InputPortDefinition {
    InputPortDefinition::required(port(id), id, "", kind).unwrap()
}

fn optional_input(id: &'static str, kind: ResourceKind) -> InputPortDefinition {
    InputPortDefinition::optional_input(port(id), id, "", kind).unwrap()
}

fn output(id: &'static str, kind: ResourceKind) -> OutputPortDefinition {
    OutputPortDefinition::new(port(id), id, "", kind).unwrap()
}

fn parameter_definition(
    id: &'static str,
    value_type: ParameterType,
    default: Option<ParameterValue>,
) -> ParameterDefinition {
    ParameterDefinition::new(parameter(id), id, "", value_type, default).unwrap()
}

fn schema(
    type_id: &'static str,
    inputs: Vec<InputPortDefinition>,
    outputs: Vec<OutputPortDefinition>,
    parameters: Vec<ParameterDefinition>,
) -> StageSchema {
    StageSchema::new(
        stage_type(type_id),
        StageSchemaVersion::V1,
        type_id,
        "Test",
        "",
        inputs,
        outputs,
        parameters,
    )
    .unwrap()
}

fn context() -> terrakit_pipeline::StageContext {
    let layout =
        RegionLayout2::new(Extent2::try_new(2, 2).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
    let region = GenerationRegion::Region2(
        layout
            .resolve(RegionRequest2::new(RegionCoord2::ZERO, LodLevel::HIGHEST))
            .unwrap(),
    );

    terrakit_pipeline::StageContext::new(GenerationSeed::new(1), region)
}

fn height_field(value: f32) -> HeightField {
    HeightField::filled(
        Extent2::try_new(3, 3).unwrap(),
        value,
        GridTransform2::identity_xz(),
        SamplingDomain::Points,
        Vector3F64::Y,
    )
    .unwrap()
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

#[test]
fn identifier_validation_covers_stable_ids() {
    let stage = StageTypeId::try_new("terrakit.height.flat").unwrap();
    let port_id = PortId::try_new("height").unwrap();
    let parameter_id = ParameterId::try_new("frequency").unwrap();

    assert_eq!(stage.as_str(), "terrakit.height.flat");
    assert_eq!(port_id.as_str(), "height");
    assert_eq!(parameter_id.as_str(), "frequency");

    assert!(StageTypeId::try_new("").is_err());
    assert!(PortId::try_new("   ").is_err());
    assert!(ParameterId::try_new("bad\nid").is_err());
    assert!(EnumValueId::try_new("x".repeat(65).into_boxed_str()).is_err());
}

#[test]
fn schema_version_validation_rejects_zero_and_accepts_one() {
    assert!(matches!(
        StageSchemaVersion::try_new(0),
        Err(DefinitionError::InvalidSchemaVersion { value: 0 })
    ));
    assert_eq!(StageSchemaVersion::try_new(1), Ok(StageSchemaVersion::V1));
    assert_eq!(StageSchemaVersion::V1.value(), 1);
}

#[test]
fn identifier_equality_ordering_and_hashing_are_stable() {
    let mut ordered = BTreeSet::new();
    ordered.insert(stage_type("test.z"));
    ordered.insert(stage_type("test.a"));

    assert_eq!(
        ordered
            .into_iter()
            .map(|id| id.as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["test.a", "test.z"]
    );

    let mut hashed = HashSet::new();
    hashed.insert(port("height"));
    hashed.insert(port("height"));
    assert_eq!(hashed.len(), 1);
}

#[test]
fn display_names_remain_independent_of_ids() {
    let schema = StageSchema::new(
        stage_type("machine.readable"),
        StageSchemaVersion::V1,
        "Human Name",
        "Human Category",
        "",
        Vec::new(),
        vec![output("out", ResourceKind::HeightField)],
        Vec::new(),
    )
    .unwrap();

    assert_eq!(schema.type_id().as_str(), "machine.readable");
    assert_eq!(schema.version(), StageSchemaVersion::V1);
    assert_eq!(schema.display_name(), "Human Name");
}

#[test]
fn stage_construction_preserves_requested_schema_version() {
    let version = StageSchemaVersion::try_new(7).unwrap();
    let construction = StageConstruction::new(StageId(9), stage_type("test.versioned"), version);

    assert_eq!(construction.stage_id(), StageId(9));
    assert_eq!(construction.stage_type().as_str(), "test.versioned");
    assert_eq!(construction.schema_version(), version);
}

#[test]
fn stage_construction_from_schema_copies_exact_version() {
    let version = StageSchemaVersion::try_new(11).unwrap();
    let schema = StageSchema::new(
        stage_type("test.schema.version"),
        version,
        "Versioned",
        "Test",
        "",
        Vec::new(),
        vec![output("out", ResourceKind::HeightField)],
        Vec::new(),
    )
    .unwrap();

    let construction = StageConstruction::from_schema(StageId(3), &schema);

    assert_eq!(construction.stage_type(), schema.type_id());
    assert_eq!(construction.schema_version(), version);
}

fn parameter_schema() -> StageSchema {
    schema(
        "test.parameters",
        Vec::new(),
        vec![output("out", ResourceKind::HeightField)],
        vec![
            parameter_definition(
                "enabled",
                ParameterType::Bool,
                Some(ParameterValue::Bool(true)),
            ),
            parameter_definition(
                "amount",
                ParameterType::F32 {
                    minimum: Some(0.0),
                    maximum: Some(1.0),
                },
                Some(ParameterValue::F32(0.25)),
            ),
            parameter_definition(
                "required_count",
                ParameterType::U64 {
                    minimum: Some(1),
                    maximum: Some(4),
                },
                None,
            ),
            parameter_definition(
                "axis",
                ParameterType::Vector3F64,
                Some(ParameterValue::Vector3F64(Vector3F64::Y)),
            ),
            parameter_definition(
                "mode",
                ParameterType::Enum {
                    options: vec![enum_option("add"), enum_option("replace")],
                },
                Some(ParameterValue::Enum(enum_value("add"))),
            ),
        ],
    )
}

#[test]
fn parameter_resolution_applies_defaults_and_overrides() {
    let schema = parameter_schema();
    let mut supplied = ParameterSet::new();
    supplied.set(parameter("required_count"), ParameterValue::U64(3));
    supplied.set(parameter("amount"), ParameterValue::F32(0.75));

    let resolved = schema.resolve_parameters(&supplied).unwrap();

    assert!(resolved.bool("enabled").unwrap());
    assert_eq!(resolved.f32("amount").unwrap(), 0.75);
    assert_eq!(resolved.u64("required_count").unwrap(), 3);
    assert_eq!(resolved.vector3_f64("axis").unwrap(), Vector3F64::Y);
    assert_eq!(resolved.enum_id("mode").unwrap().as_str(), "add");
}

#[test]
fn parameter_resolution_rejects_unknown_and_missing_values() {
    let schema = parameter_schema();
    let mut supplied = ParameterSet::new();
    supplied.set(parameter("unknown"), ParameterValue::Bool(false));

    assert!(matches!(
        schema.resolve_parameters(&supplied),
        Err(ParameterError::UnknownParameter { .. })
    ));

    assert!(matches!(
        schema.resolve_parameters(&ParameterSet::new()),
        Err(ParameterError::MissingRequiredParameter { .. })
    ));
}

#[test]
fn parameter_resolution_rejects_type_mismatch_nonfinite_and_bounds() {
    let schema = parameter_schema();

    let mut wrong_type = ParameterSet::new();
    wrong_type.set(parameter("required_count"), ParameterValue::U64(1));
    wrong_type.set(parameter("amount"), ParameterValue::F64(0.5));
    assert!(matches!(
        schema.resolve_parameters(&wrong_type),
        Err(ParameterError::WrongValueType {
            expected: ParameterKind::F32,
            actual: ParameterKind::F64,
            ..
        })
    ));

    let mut nonfinite = ParameterSet::new();
    nonfinite.set(parameter("required_count"), ParameterValue::U64(1));
    nonfinite.set(parameter("amount"), ParameterValue::F32(f32::NAN));
    assert!(matches!(
        schema.resolve_parameters(&nonfinite),
        Err(ParameterError::NonFiniteNumericValue { .. })
    ));

    let mut below = ParameterSet::new();
    below.set(parameter("required_count"), ParameterValue::U64(0));
    assert!(matches!(
        schema.resolve_parameters(&below),
        Err(ParameterError::BelowMinimum { .. })
    ));

    let mut above = ParameterSet::new();
    above.set(parameter("required_count"), ParameterValue::U64(5));
    assert!(matches!(
        schema.resolve_parameters(&above),
        Err(ParameterError::AboveMaximum { .. })
    ));
}

#[test]
fn parameter_definition_rejects_invalid_ranges_and_enum_defaults() {
    assert!(matches!(
        ParameterDefinition::new(
            parameter("bad_range"),
            "Bad Range",
            "",
            ParameterType::F64 {
                minimum: Some(2.0),
                maximum: Some(1.0),
            },
            None,
        ),
        Err(DefinitionError::InvalidNumericRange { .. })
    ));

    assert!(matches!(
        ParameterDefinition::new(
            parameter("bad_enum"),
            "Bad Enum",
            "",
            ParameterType::Enum {
                options: vec![enum_option("one")],
            },
            Some(ParameterValue::Enum(enum_value("two"))),
        ),
        Err(DefinitionError::InvalidDefault { .. })
    ));
}

#[test]
fn parameter_resolution_rejects_unknown_enum_selection() {
    let schema = parameter_schema();
    let mut supplied = ParameterSet::new();
    supplied.set(parameter("required_count"), ParameterValue::U64(1));
    supplied.set(
        parameter("mode"),
        ParameterValue::Enum(enum_value("multiply")),
    );

    assert!(matches!(
        schema.resolve_parameters(&supplied),
        Err(ParameterError::UnknownEnumValue { .. })
    ));
}

#[test]
fn schema_validation_and_lookup_work() {
    let schema = StageSchema::new(
        stage_type("test.schema"),
        StageSchemaVersion::V1,
        "Schema",
        "Test",
        "",
        vec![input("height", ResourceKind::HeightField)],
        vec![output("height", ResourceKind::HeightField)],
        vec![parameter_definition(
            "amount",
            ParameterType::F32 {
                minimum: None,
                maximum: None,
            },
            Some(ParameterValue::F32(0.0)),
        )],
    )
    .unwrap();

    assert!(schema.input(&port("height")).is_some());
    assert!(schema.output(&port("height")).is_some());
    assert!(schema.parameter(&parameter("amount")).is_some());
    assert_eq!(schema.inputs()[0].id().as_str(), "height");
    assert_eq!(schema.outputs()[0].id().as_str(), "height");
    assert_eq!(schema.parameters()[0].id().as_str(), "amount");
}

#[test]
fn schema_validation_rejects_duplicates() {
    assert!(matches!(
        StageSchema::new(
            stage_type("test.duplicate.input"),
            StageSchemaVersion::V1,
            "Duplicate",
            "Test",
            "",
            vec![
                input("height", ResourceKind::HeightField),
                input("height", ResourceKind::HeightField),
            ],
            Vec::new(),
            Vec::new(),
        ),
        Err(DefinitionError::DuplicateInputPort { .. })
    ));

    assert!(matches!(
        StageSchema::new(
            stage_type("test.duplicate.output"),
            StageSchemaVersion::V1,
            "Duplicate",
            "Test",
            "",
            Vec::new(),
            vec![
                output("height", ResourceKind::HeightField),
                output("height", ResourceKind::HeightField),
            ],
            Vec::new(),
        ),
        Err(DefinitionError::DuplicateOutputPort { .. })
    ));

    assert!(matches!(
        StageSchema::new(
            stage_type("test.duplicate.parameter"),
            StageSchemaVersion::V1,
            "Duplicate",
            "Test",
            "",
            Vec::new(),
            Vec::new(),
            vec![
                parameter_definition(
                    "amount",
                    ParameterType::Bool,
                    Some(ParameterValue::Bool(true))
                ),
                parameter_definition(
                    "amount",
                    ParameterType::Bool,
                    Some(ParameterValue::Bool(false))
                ),
            ],
        ),
        Err(DefinitionError::DuplicateParameter { .. })
    ));
}

#[test]
fn registry_registration_retrieval_and_order_are_deterministic() {
    let mut registry = StageRegistry::new();
    assert!(registry.is_empty());

    registry
        .register(SourceDefinition::new("test.z", "height"))
        .unwrap();
    registry
        .register(SourceDefinition::new("test.a", "height"))
        .unwrap();

    assert_eq!(registry.len(), 2);
    assert!(registry.definition(&stage_type("test.a")).is_some());
    assert_eq!(
        registry
            .schemas()
            .map(|schema| schema.type_id().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["test.a", "test.z"]
    );

    assert!(
        registry
            .register(SourceDefinition::new("test.a", "height"))
            .is_err()
    );
}

#[test]
fn required_optional_and_unknown_bindings_are_validated() {
    let schema = schema(
        "test.bindings",
        vec![
            input("height", ResourceKind::HeightField),
            optional_input("mask", ResourceKind::DensityField),
        ],
        vec![output("out", ResourceKind::HeightField)],
        Vec::new(),
    );

    let bindings = terrakit_pipeline::StageBindings::new()
        .with_input(port("height"), ResourceKey(1))
        .unwrap()
        .with_output(port("out"), ResourceKey(2))
        .unwrap();
    let validated = ValidatedStageBindings::validate(&schema, &bindings).unwrap();
    assert_eq!(validated.required_input("height").unwrap(), ResourceKey(1));
    assert_eq!(validated.input("mask"), None);
    assert_eq!(validated.output("out").unwrap(), ResourceKey(2));

    let missing_input = terrakit_pipeline::StageBindings::new()
        .with_output(port("out"), ResourceKey(2))
        .unwrap();
    assert!(matches!(
        ValidatedStageBindings::validate(&schema, &missing_input),
        Err(BindingError::MissingRequiredInput { .. })
    ));

    let missing_output = terrakit_pipeline::StageBindings::new()
        .with_input(port("height"), ResourceKey(1))
        .unwrap();
    assert!(matches!(
        ValidatedStageBindings::validate(&schema, &missing_output),
        Err(BindingError::MissingOutput { .. })
    ));

    let unknown_input = terrakit_pipeline::StageBindings::new()
        .with_input(port("other"), ResourceKey(1))
        .unwrap()
        .with_output(port("out"), ResourceKey(2))
        .unwrap();
    assert!(matches!(
        ValidatedStageBindings::validate(&schema, &unknown_input),
        Err(BindingError::UnknownInputPort { .. })
    ));
}

#[test]
fn duplicate_and_alias_bindings_are_rejected() {
    let mut duplicate_input = terrakit_pipeline::StageBindings::new();
    duplicate_input
        .bind_input(port("height"), ResourceKey(1))
        .unwrap();
    assert!(matches!(
        duplicate_input.bind_input(port("height"), ResourceKey(2)),
        Err(BindingError::DuplicateInputBinding { .. })
    ));

    let schema = schema(
        "test.output.resources",
        vec![input("height", ResourceKind::HeightField)],
        vec![
            output("a", ResourceKind::HeightField),
            output("b", ResourceKind::HeightField),
        ],
        Vec::new(),
    );
    let duplicate_outputs = terrakit_pipeline::StageBindings::new()
        .with_input(port("height"), ResourceKey(1))
        .unwrap()
        .with_output(port("a"), ResourceKey(2))
        .unwrap()
        .with_output(port("b"), ResourceKey(2))
        .unwrap();
    assert!(matches!(
        ValidatedStageBindings::validate(&schema, &duplicate_outputs),
        Err(BindingError::DuplicateOutputResource { .. })
    ));

    let alias = terrakit_pipeline::StageBindings::new()
        .with_input(port("height"), ResourceKey(1))
        .unwrap()
        .with_output(port("a"), ResourceKey(1))
        .unwrap()
        .with_output(port("b"), ResourceKey(2))
        .unwrap();
    assert!(matches!(
        ValidatedStageBindings::validate(&schema, &alias),
        Err(BindingError::InputOutputAlias { .. })
    ));
}

#[test]
fn same_resource_can_be_bound_to_multiple_inputs() {
    let schema = schema(
        "test.shared.input",
        vec![
            input("left", ResourceKind::HeightField),
            input("right", ResourceKind::HeightField),
        ],
        vec![output("out", ResourceKind::HeightField)],
        Vec::new(),
    );
    let bindings = terrakit_pipeline::StageBindings::new()
        .with_input(port("left"), ResourceKey(1))
        .unwrap()
        .with_input(port("right"), ResourceKey(1))
        .unwrap()
        .with_output(port("out"), ResourceKey(2))
        .unwrap();

    assert!(ValidatedStageBindings::validate(&schema, &bindings).is_ok());
}

#[test]
fn assembler_accepts_ordered_source_then_consumer() {
    let registry = test_registry();
    let mut assembler = TerrainPipelineAssembler::new(&registry);
    assembler
        .add_stage(source_construction(StageId(1), ResourceKey(100)))
        .unwrap();
    assembler
        .add_stage(copy_construction(
            StageId(2),
            ResourceKey(100),
            ResourceKey(101),
        ))
        .unwrap();
    let mut pipeline = assembler.finish();
    let mut resources = ResourceSet::new();

    pipeline.execute(&context(), &mut resources).unwrap();

    assert!(resources.height_field(ResourceKey(100)).is_some());
    assert!(resources.height_field(ResourceKey(101)).is_some());
}

#[test]
fn assembler_rejects_out_of_order_input_without_reordering() {
    let registry = test_registry();
    let mut assembler = TerrainPipelineAssembler::new(&registry);

    let error = assembler
        .add_stage(copy_construction(
            StageId(2),
            ResourceKey(100),
            ResourceKey(101),
        ))
        .unwrap_err();

    assert!(matches!(
        error,
        PipelineAssemblyError::MissingInputResource {
            stage_id: StageId(2),
            key: ResourceKey(100),
            ..
        }
    ));
}

#[test]
fn assembler_allows_branching_from_one_producer_to_two_readers() {
    let registry = test_registry();
    let mut assembler = TerrainPipelineAssembler::new(&registry);
    assembler
        .add_stage(source_construction(StageId(1), ResourceKey(100)))
        .unwrap();
    assembler
        .add_stage(copy_construction(
            StageId(2),
            ResourceKey(100),
            ResourceKey(101),
        ))
        .unwrap();
    assembler
        .add_stage(copy_construction(
            StageId(3),
            ResourceKey(100),
            ResourceKey(102),
        ))
        .unwrap();
    let mut pipeline = assembler.finish();
    let mut resources = ResourceSet::new();

    pipeline.execute(&context(), &mut resources).unwrap();

    assert!(resources.height_field(ResourceKey(100)).is_some());
    assert!(resources.height_field(ResourceKey(101)).is_some());
    assert!(resources.height_field(ResourceKey(102)).is_some());
}

#[test]
fn assembler_rejects_duplicate_producers_kind_mismatch_and_duplicate_stage_ids() {
    let duplicate_output_registry = test_registry();
    let mut duplicate_output = TerrainPipelineAssembler::new(&duplicate_output_registry);
    duplicate_output
        .add_stage(source_construction(StageId(1), ResourceKey(100)))
        .unwrap();
    assert!(matches!(
        duplicate_output.add_stage(source_construction(StageId(2), ResourceKey(100))),
        Err(PipelineAssemblyError::OutputResourceAlreadyAssigned { .. })
    ));

    let wrong_kind_registry = test_registry();
    let mut wrong_kind = TerrainPipelineAssembler::new(&wrong_kind_registry);
    wrong_kind
        .add_stage(mesh_source_construction(StageId(1), ResourceKey(100)))
        .unwrap();
    assert!(matches!(
        wrong_kind.add_stage(copy_construction(
            StageId(2),
            ResourceKey(100),
            ResourceKey(101),
        )),
        Err(PipelineAssemblyError::WrongInputResourceKind { .. })
    ));

    let duplicate_stage_registry = test_registry();
    let mut duplicate_stage = TerrainPipelineAssembler::new(&duplicate_stage_registry);
    duplicate_stage
        .add_stage(source_construction(StageId(1), ResourceKey(100)))
        .unwrap();
    assert!(matches!(
        duplicate_stage.add_stage(source_construction(StageId(1), ResourceKey(101))),
        Err(PipelineAssemblyError::DuplicateStageId { .. })
    ));
}

#[test]
fn assembler_rejects_unknown_type_factory_failure_and_contract_mismatch() {
    let unknown_registry = test_registry();
    let mut unknown = TerrainPipelineAssembler::new(&unknown_registry);
    assert!(matches!(
        unknown.add_stage(StageConstruction::new(
            StageId(1),
            stage_type("test.unknown"),
            StageSchemaVersion::V1,
        )),
        Err(PipelineAssemblyError::UnknownStageType { .. })
    ));

    let registry = failing_registry();
    let mut failing = TerrainPipelineAssembler::new(&registry);
    assert!(matches!(
        failing.add_stage(
            StageConstruction::new(
                StageId(1),
                stage_type("test.failing"),
                StageSchemaVersion::V1,
            )
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_output(port("height"), ResourceKey(100))
                    .unwrap()
            )
        ),
        Err(PipelineAssemblyError::StageBuild { .. })
    ));

    let registry = contract_mismatch_registry();
    let mut mismatch = TerrainPipelineAssembler::new(&registry);
    assert!(matches!(
        mismatch.add_stage(source_construction_for_type(
            StageId(1),
            stage_type("test.mismatch"),
            ResourceKey(100),
        )),
        Err(PipelineAssemblyError::StageContractMismatch { .. })
    ));
}

#[test]
fn failed_add_stage_is_atomic() {
    let registry = test_registry();
    let mut assembler = TerrainPipelineAssembler::new(&registry);

    assert!(
        assembler
            .add_stage(copy_construction(
                StageId(2),
                ResourceKey(100),
                ResourceKey(101),
            ))
            .is_err()
    );

    assembler
        .add_stage(source_construction(StageId(1), ResourceKey(100)))
        .unwrap();
    assembler
        .add_stage(copy_construction(
            StageId(2),
            ResourceKey(100),
            ResourceKey(101),
        ))
        .unwrap();
}

#[test]
fn schema_version_mismatch_fails_before_factory_and_remains_atomic() {
    let factory_calls = Arc::new(AtomicUsize::new(0));
    let mut registry = StageRegistry::new();
    registry
        .register(CountingSourceDefinition::new(
            "test.versioned.source",
            Arc::clone(&factory_calls),
        ))
        .unwrap();
    let mut assembler = TerrainPipelineAssembler::new(&registry);
    let requested = StageSchemaVersion::try_new(2).unwrap();
    let stage_type = stage_type("test.versioned.source");

    let mismatch = StageConstruction::new(StageId(42), stage_type.clone(), requested)
        .with_bindings(
            terrakit_pipeline::StageBindings::new()
                .with_output(port("height"), ResourceKey(700))
                .unwrap(),
        );
    let error = assembler.add_stage(mismatch).unwrap_err();

    assert!(matches!(
        error,
        PipelineAssemblyError::SchemaVersionMismatch {
            stage_id: StageId(42),
            requested: version,
            available: StageSchemaVersion::V1,
            ..
        } if version == requested
    ));
    assert_eq!(factory_calls.load(Ordering::SeqCst), 0);

    assembler
        .add_stage(
            StageConstruction::new(StageId(42), stage_type, StageSchemaVersion::V1).with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_output(port("height"), ResourceKey(700))
                    .unwrap(),
            ),
        )
        .unwrap();

    assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
}

fn test_registry() -> StageRegistry {
    let mut registry = StageRegistry::new();
    registry
        .register(SourceDefinition::new("test.source", "height"))
        .unwrap();
    registry
        .register(MeshSourceDefinition::new("test.mesh_source"))
        .unwrap();
    registry.register(CopyDefinition::new()).unwrap();
    registry
}

fn failing_registry() -> StageRegistry {
    let mut registry = StageRegistry::new();
    registry.register(FailingDefinition::new()).unwrap();
    registry
}

fn contract_mismatch_registry() -> StageRegistry {
    let mut registry = StageRegistry::new();
    registry
        .register(ContractMismatchDefinition::new())
        .unwrap();
    registry
}

fn source_construction(stage_id: StageId, output_key: ResourceKey) -> StageConstruction {
    source_construction_for_type(stage_id, stage_type("test.source"), output_key)
}

fn mesh_source_construction(stage_id: StageId, output_key: ResourceKey) -> StageConstruction {
    StageConstruction::new(
        stage_id,
        stage_type("test.mesh_source"),
        StageSchemaVersion::V1,
    )
    .with_bindings(
        terrakit_pipeline::StageBindings::new()
            .with_output(port("mesh"), output_key)
            .unwrap(),
    )
}

fn source_construction_for_type(
    stage_id: StageId,
    stage_type: StageTypeId,
    output_key: ResourceKey,
) -> StageConstruction {
    StageConstruction::new(stage_id, stage_type, StageSchemaVersion::V1).with_bindings(
        terrakit_pipeline::StageBindings::new()
            .with_output(port("height"), output_key)
            .unwrap(),
    )
}

fn copy_construction(
    stage_id: StageId,
    input_key: ResourceKey,
    output_key: ResourceKey,
) -> StageConstruction {
    StageConstruction::new(stage_id, stage_type("test.copy"), StageSchemaVersion::V1).with_bindings(
        terrakit_pipeline::StageBindings::new()
            .with_input(port("source"), input_key)
            .unwrap()
            .with_output(port("height"), output_key)
            .unwrap(),
    )
}

struct SourceDefinition {
    schema: StageSchema,
}

impl SourceDefinition {
    fn new(type_id: &'static str, output_port: &'static str) -> Self {
        Self {
            schema: schema(
                type_id,
                Vec::new(),
                vec![output(output_port, ResourceKind::HeightField)],
                Vec::new(),
            ),
        }
    }
}

struct CountingSourceDefinition {
    schema: StageSchema,
    factory_calls: Arc<AtomicUsize>,
}

impl CountingSourceDefinition {
    fn new(type_id: &'static str, factory_calls: Arc<AtomicUsize>) -> Self {
        Self {
            schema: schema(
                type_id,
                Vec::new(),
                vec![output("height", ResourceKind::HeightField)],
                Vec::new(),
            ),
            factory_calls,
        }
    }
}

impl StageDefinition for CountingSourceDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        _parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn TerrainStage>, StageBuildError> {
        self.factory_calls.fetch_add(1, Ordering::SeqCst);

        Ok(Box::new(ProduceHeightStage {
            id: stage_id,
            output: bindings.output("height")?,
            products: [ResourceProduct::create(
                bindings.output("height")?,
                ResourceKind::HeightField,
            )],
        }))
    }
}

impl StageDefinition for SourceDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        _parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn TerrainStage>, StageBuildError> {
        Ok(Box::new(ProduceHeightStage {
            id: stage_id,
            output: bindings.output("height")?,
            products: [ResourceProduct::create(
                bindings.output("height")?,
                ResourceKind::HeightField,
            )],
        }))
    }
}

struct MeshSourceDefinition {
    schema: StageSchema,
}

impl MeshSourceDefinition {
    fn new(type_id: &'static str) -> Self {
        Self {
            schema: schema(
                type_id,
                Vec::new(),
                vec![output("mesh", ResourceKind::Mesh)],
                Vec::new(),
            ),
        }
    }
}

impl StageDefinition for MeshSourceDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        _parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn TerrainStage>, StageBuildError> {
        Ok(Box::new(ProduceMeshStage {
            id: stage_id,
            output: bindings.output("mesh")?,
            products: [ResourceProduct::create(
                bindings.output("mesh")?,
                ResourceKind::Mesh,
            )],
        }))
    }
}

struct CopyDefinition {
    schema: StageSchema,
}

impl CopyDefinition {
    fn new() -> Self {
        Self {
            schema: schema(
                "test.copy",
                vec![input("source", ResourceKind::HeightField)],
                vec![output("height", ResourceKind::HeightField)],
                Vec::new(),
            ),
        }
    }
}

impl StageDefinition for CopyDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        _parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn TerrainStage>, StageBuildError> {
        Ok(Box::new(CopyHeightStage {
            id: stage_id,
            input: bindings.required_input("source")?,
            output: bindings.output("height")?,
            requirements: [ResourceRequirement::required(
                bindings.required_input("source")?,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],
            products: [ResourceProduct::create(
                bindings.output("height")?,
                ResourceKind::HeightField,
            )],
        }))
    }
}

struct FailingDefinition {
    schema: StageSchema,
}

impl FailingDefinition {
    fn new() -> Self {
        Self {
            schema: schema(
                "test.failing",
                Vec::new(),
                vec![output("height", ResourceKind::HeightField)],
                Vec::new(),
            ),
        }
    }
}

impl StageDefinition for FailingDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        _stage_id: StageId,
        _parameters: &ResolvedParameterSet,
        _bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn TerrainStage>, StageBuildError> {
        Err(StageBuildError::new("known factory failure"))
    }
}

struct ContractMismatchDefinition {
    schema: StageSchema,
}

impl ContractMismatchDefinition {
    fn new() -> Self {
        Self {
            schema: schema(
                "test.mismatch",
                Vec::new(),
                vec![output("height", ResourceKind::HeightField)],
                Vec::new(),
            ),
        }
    }
}

impl StageDefinition for ContractMismatchDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        _parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn TerrainStage>, StageBuildError> {
        Ok(Box::new(ProduceHeightStage {
            id: stage_id,
            output: bindings.output("height")?,
            products: [ResourceProduct::replace(
                bindings.output("height")?,
                ResourceKind::HeightField,
            )],
        }))
    }
}

struct ProduceHeightStage {
    id: StageId,
    output: ResourceKey,
    products: [ResourceProduct; 1],
}

impl TerrainStage for ProduceHeightStage {
    fn id(&self) -> StageId {
        self.id
    }

    fn name(&self) -> &str {
        "produce_height"
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        _context: &terrakit_pipeline::StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), terrakit_pipeline::StageError> {
        resources.insert(
            self.output,
            TerrainResource::HeightField(height_field(self.id.0 as f32)),
        );

        Ok(())
    }
}

struct ProduceMeshStage {
    id: StageId,
    output: ResourceKey,
    products: [ResourceProduct; 1],
}

impl TerrainStage for ProduceMeshStage {
    fn id(&self) -> StageId {
        self.id
    }

    fn name(&self) -> &str {
        "produce_mesh"
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        _context: &terrakit_pipeline::StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), terrakit_pipeline::StageError> {
        resources.insert(self.output, TerrainResource::Mesh(mesh()));

        Ok(())
    }
}

struct CopyHeightStage {
    id: StageId,
    input: ResourceKey,
    output: ResourceKey,
    requirements: [ResourceRequirement; 1],
    products: [ResourceProduct; 1],
}

impl TerrainStage for CopyHeightStage {
    fn id(&self) -> StageId {
        self.id
    }

    fn name(&self) -> &str {
        "copy_height"
    }

    fn requirements(&self) -> &[ResourceRequirement] {
        &self.requirements
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        _context: &terrakit_pipeline::StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), terrakit_pipeline::StageError> {
        let Some(source) = resources.height_field(self.input).cloned() else {
            return Err(terrakit_pipeline::StageError::new("source missing"));
        };
        resources.insert(self.output, TerrainResource::HeightField(source));

        Ok(())
    }
}
