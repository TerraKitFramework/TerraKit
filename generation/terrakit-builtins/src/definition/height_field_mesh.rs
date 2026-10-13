//! Schema and factory for the built-in heightfield-mesh stage.

use terrakit_pipeline::{
    DefinitionError, ParameterType, ParameterValue, ResolvedParameterSet, ResourceKind,
    StageBuildError, StageDefinition, StageId, StageSchema, StageSchemaVersion,
    ValidatedStageBindings,
};

use crate::HeightFieldMeshStage;

use super::{input, output, parameter, schema};

/// Stage definition for converting a height field into a terrain mesh.
pub struct HeightFieldMeshDefinition {
    schema: StageSchema,
}

impl HeightFieldMeshDefinition {
    /// Stable stage type ID for heightfield mesh generation.
    pub const TYPE_ID: &'static str = "terrakit.mesh.height_field";

    /// Compatible schema revision for this built-in definition.
    pub const SCHEMA_VERSION: StageSchemaVersion = StageSchemaVersion::V1;

    /// Creates the built-in heightfield mesh stage definition.
    pub fn new() -> Result<Self, DefinitionError> {
        Ok(Self {
            schema: schema(
                Self::TYPE_ID,
                Self::SCHEMA_VERSION,
                "Heightfield Mesh",
                "Mesh",
                "Converts a height field into an indexed terrain mesh.",
                vec![input(
                    "height",
                    "Height",
                    "Source height field.",
                    ResourceKind::HeightField,
                )?],
                vec![output(
                    "mesh",
                    "Mesh",
                    "Created terrain mesh.",
                    ResourceKind::Mesh,
                )?],
                vec![
                    parameter(
                        "generate_normals",
                        "Generate Normals",
                        "Generate smooth vertex normals.",
                        ParameterType::Bool,
                        Some(ParameterValue::Bool(true)),
                    )?,
                    parameter(
                        "generate_texcoords",
                        "Generate Texcoords",
                        "Generate regular grid texture coordinates.",
                        ParameterType::Bool,
                        Some(ParameterValue::Bool(true)),
                    )?,
                ],
            )?,
        })
    }
}

impl StageDefinition for HeightFieldMeshDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn terrakit_pipeline::TerrainStage>, StageBuildError> {
        let stage = HeightFieldMeshStage::new(
            stage_id,
            bindings.required_input("height")?,
            bindings.output("mesh")?,
        )
        .with_normals(parameters.bool("generate_normals")?)
        .with_texcoords(parameters.bool("generate_texcoords")?);

        Ok(Box::new(stage))
    }
}
