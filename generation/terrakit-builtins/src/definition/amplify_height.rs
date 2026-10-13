//! Schema and factory for the built-in amplify-height stage.

use terrakit_pipeline::{
    DefinitionError, EnumValueId, ParameterType, ParameterValue, ResolvedParameterSet,
    ResourceKind, StageBuildError, StageDefinition, StageId, StageSchema, StageSchemaVersion,
    ValidatedStageBindings,
};

use crate::{AmplifyHeightStage, HeightAmplifyMode};

use super::{enum_option, enum_value_id, input, output, parameter, schema};

/// Stage definition for amplifying existing height values across a height field
pub struct AmplifyHeightDefinition {
    schema: StageSchema,
}

impl AmplifyHeightDefinition {
    /// Stable stage type ID for height amplification.
    pub const TYPE_ID: &'static str = "terrakit.height.amplify";

    /// Compatible schema revision for this built-in definition.
    pub const SCHEMA_VERSION: StageSchemaVersion = StageSchemaVersion::V1;

    /// Creates the built-in height amplify stage definition.
    pub fn new() -> Result<Self, DefinitionError> {
        Ok(Self {
            schema: schema(
                Self::TYPE_ID,
                Self::SCHEMA_VERSION,
                "Amplify Height",
                "Height",
                "Reads a source height field and amplifies height values according to given parameters",
                vec![input(
                    "source",
                    "Source",
                    "Source height field to read.",
                    ResourceKind::HeightField,
                )?],
                vec![output(
                    "out_height",
                    "Height",
                    "Created amplified height field.",
                    ResourceKind::HeightField,
                )?],
                vec![
                    parameter(
                        "threshold_min",
                        "Minimum Threshold",
                        "Minimum height value affected by amplification.",
                        ParameterType::F32 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F32(0.5)),
                    )?,
                    parameter(
                        "threshold_max",
                        "Maximum Threshold",
                        "Maximum height value affected by amplification.",
                        ParameterType::F32 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F32(10.0)),
                    )?,
                    parameter(
                        "amplify_mode",
                        "Amplification Mode",
                        "Whether amplification is additive or multiplicative.",
                        ParameterType::Enum {
                            options: vec![
                                enum_option(
                                    "add",
                                    "Additive",
                                    "Amplification value is added to the current height.",
                                )?,
                                enum_option(
                                    "multiply",
                                    "Multiplicative",
                                    "Current height is multiplied by the amplification value.",
                                )?,
                            ],
                        },
                        Some(ParameterValue::Enum(enum_value_id("add")?)),
                    )?,
                    parameter(
                        "amplify_value",
                        "Amplification Value",
                        "Value by which height is amplified.",
                        ParameterType::F32 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F32(5.00)),
                    )?,
                    parameter(
                        "amplify_limit",
                        "Amplification Limit",
                        "Highest or lowest height that can be achieved by the amplification step.",
                        ParameterType::F32 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F32(10.00)),
                    )?,
                ],
            )?,
        })
    }
}

impl StageDefinition for AmplifyHeightDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn terrakit_pipeline::TerrainStage>, StageBuildError> {
        let amplify_mode = amplify_mode(parameters.enum_id("amplify_mode")?)?;
        let threshold_min = parameters.f32("threshold_min")?;
        let threshold_max = parameters.f32("threshold_max")?;
        let amplify_value = parameters.f32("amplify_value")?;
        let amplify_limit = parameters.f32("amplify_limit")?;
        let stage = AmplifyHeightStage::new(
            stage_id,
            bindings.required_input("source")?,
            bindings.output("out_height")?,
            amplify_mode,
            threshold_min,
            threshold_max,
            amplify_value,
            amplify_limit,
        )
        .map_err(|source| {
            StageBuildError::from_stage("failed to build Amplify Height stage", source)
        })?;

        Ok(Box::new(stage))
    }
}

fn amplify_mode(value: &EnumValueId) -> Result<HeightAmplifyMode, StageBuildError> {
    match value.as_str() {
        "add" => Ok(HeightAmplifyMode::Add),
        "multiply" => Ok(HeightAmplifyMode::Multiply),
        other => Err(StageBuildError::new(format!(
            "unknown height amplify mode enum id '{other}'"
        ))),
    }
}
