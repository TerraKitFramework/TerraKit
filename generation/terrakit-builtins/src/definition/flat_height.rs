//! Schema and factory for the built-in flat-height stage.

use terrakit_core::Vector3F64;
use terrakit_pipeline::{
    DefinitionError, ParameterType, ParameterValue, ResolvedParameterSet, ResourceKind,
    StageBuildError, StageDefinition, StageId, StageSchema, StageSchemaVersion,
    ValidatedStageBindings,
};

use crate::FlatHeightStage;

use super::{output, parameter, schema};

/// Stage definition for creating a constant height field.
pub struct FlatHeightDefinition {
    schema: StageSchema,
}

impl FlatHeightDefinition {
    /// Stable stage type ID for flat height creation.
    pub const TYPE_ID: &'static str = "terrakit.height.flat";

    /// Compatible schema revision for this built-in definition.
    pub const SCHEMA_VERSION: StageSchemaVersion = StageSchemaVersion::V1;

    /// Creates the built-in flat height stage definition.
    pub fn new() -> Result<Self, DefinitionError> {
        Ok(Self {
            schema: schema(
                Self::TYPE_ID,
                Self::SCHEMA_VERSION,
                "Flat Height",
                "Height",
                "Creates a height field filled with one elevation value.",
                Vec::new(),
                vec![output(
                    "height",
                    "Height",
                    "Created height field resource.",
                    ResourceKind::HeightField,
                )?],
                vec![
                    parameter(
                        "elevation",
                        "Elevation",
                        "Constant elevation assigned to every height sample.",
                        ParameterType::F32 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F32(0.0)),
                    )?,
                    parameter(
                        "height_axis",
                        "Height Axis",
                        "World-space axis used for height displacement.",
                        ParameterType::Vector3F64,
                        Some(ParameterValue::Vector3F64(Vector3F64::Y)),
                    )?,
                ],
            )?,
        })
    }
}

impl StageDefinition for FlatHeightDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn terrakit_pipeline::TerrainStage>, StageBuildError> {
        let stage = FlatHeightStage::new(
            stage_id,
            bindings.output("height")?,
            parameters.f32("elevation")?,
            parameters.vector3_f64("height_axis")?,
        )
        .map_err(|source| {
            StageBuildError::from_stage("failed to build Flat Height stage", source)
        })?;

        Ok(Box::new(stage))
    }
}
