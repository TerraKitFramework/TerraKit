use terrakit_pipeline::{
    OutputPortDefinition, ResourceKey, ResourceKind, StageConstruction, StageId, StageSchema,
    StageSchemaVersion, StageTypeId,
};

fn stage_type(value: &'static str) -> StageTypeId {
    StageTypeId::try_new(value).unwrap()
}

#[test]
fn versioned_stage_schema_and_construction_are_public() {
    let version = StageSchemaVersion::try_new(1).unwrap();
    let schema = StageSchema::new(
        stage_type("example.height"),
        version,
        "Example Height",
        "Example",
        "",
        Vec::new(),
        vec![
            OutputPortDefinition::new(
                terrakit_pipeline::PortId::try_new("height").unwrap(),
                "Height",
                "",
                ResourceKind::HeightField,
            )
            .unwrap(),
        ],
        Vec::new(),
    )
    .unwrap();

    let saved_node =
        StageConstruction::new(StageId(10), schema.type_id().clone(), schema.version());
    let fresh_node = StageConstruction::from_schema(StageId(11), &schema);

    assert_eq!(schema.version(), version);
    assert_eq!(saved_node.schema_version(), version);
    assert_eq!(fresh_node.stage_type(), schema.type_id());
    assert_eq!(fresh_node.schema_version(), version);
    assert_eq!(ResourceKey::HEIGHT, ResourceKey(1));
}
