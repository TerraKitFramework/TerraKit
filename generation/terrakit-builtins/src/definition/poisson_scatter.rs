//! Schema and factory for the built-in Poisson scatter stage.

use terrakit_pipeline::{
    DefinitionError, ParameterType, ParameterValue, ResolvedParameterSet, ResourceKind,
    StageBuildError, StageDefinition, StageId, StageSchema, StageSchemaVersion,
    ValidatedStageBindings,
};

use crate::PoissonScatterStage;
use crate::scattering::PoissonScatterConfig;

use super::{input, output, parameter, schema};

/// Schema and factory for the Poisson scatter stage.
pub struct PoissonScatterDefinition {
    schema: StageSchema,
}

impl PoissonScatterDefinition {
    /// Stable identifier of the built-in Poisson scattering stage.
    pub const TYPE_ID: &'static str = "terrakit.scatter.poisson";
    /// Version of the stage parameter schema.
    pub const SCHEMA_VERSION: StageSchemaVersion = StageSchemaVersion::V1;

    /// Creates the discoverable stage definition.
    pub fn new() -> Result<Self, DefinitionError> {
        Ok(Self {
            schema: schema(
                Self::TYPE_ID,
                Self::SCHEMA_VERSION,
                "Poisson Scatter",
                "Scattering",
                "Places objects across a height field using Poisson-disc sampling.",
                vec![input(
                    "height",
                    "Height",
                    "Height field used as the surface for scattering.",
                    ResourceKind::HeightField,
                )?],
                vec![output(
                    "scatter",
                    "Scatter",
                    "Generated object placement points.",
                    ResourceKind::ScatterPoints,
                )?],
                vec![
                    parameter(
                        "radius",
                        "Radius",
                        "Minimum distance between scattered objects.",
                        ParameterType::F32 {
                            minimum: Some(0.01),
                            maximum: None,
                        },
                        Some(ParameterValue::F32(2.0)),
                    )?,
                    parameter(
                        "attempts",
                        "Attempts",
                        "Number of placement attempts made for each accepted point.",
                        ParameterType::U64 {
                            minimum: Some(1),
                            maximum: Some(10_000),
                        },
                        Some(ParameterValue::U64(30)),
                    )?,
                    parameter(
                        "prototype_id",
                        "Prototype ID",
                        "Identifier of the object prototype to scatter.",
                        ParameterType::U64 {
                            minimum: Some(0),
                            maximum: Some(u32::MAX as u64),
                        },
                        Some(ParameterValue::U64(0)),
                    )?,
                    parameter(
                        "min_scale",
                        "Minimum Scale",
                        "Minimum random scale applied to scattered objects.",
                        ParameterType::F32 {
                            minimum: Some(0.01),
                            maximum: None,
                        },
                        Some(ParameterValue::F32(1.0)),
                    )?,
                    parameter(
                        "max_scale",
                        "Maximum Scale",
                        "Maximum random scale applied to scattered objects.",
                        ParameterType::F32 {
                            minimum: Some(0.01),
                            maximum: None,
                        },
                        Some(ParameterValue::F32(1.0)),
                    )?,
                    parameter(
                        "random_rotation",
                        "Random Rotation",
                        "Randomises the Y rotation of scattered objects.",
                        ParameterType::Bool,
                        Some(ParameterValue::Bool(true)),
                    )?,
                ],
            )?,
        })
    }
}

impl StageDefinition for PoissonScatterDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn terrakit_pipeline::TerrainStage>, StageBuildError> {
        let attempts = parameters.u64("attempts")?;

        let attempts = u32::try_from(attempts).map_err(|_| {
            StageBuildError::new("attempts must fit within a 32-bit unsigned integer")
        })?;

        let prototype_id = parameters.u64("prototype_id")?;

        let prototype_id = u32::try_from(prototype_id).map_err(|_| {
            StageBuildError::new("prototype_id must fit within a 32-bit unsigned integer")
        })?;

        let stage = PoissonScatterStage::new(
            stage_id,
            bindings.input("height").ok_or_else(|| {
                StageBuildError::new("Poisson Scatter is missing the required height input binding")
            })?,
            bindings.output("scatter")?,
            PoissonScatterConfig {
                radius: parameters.f32("radius")?,
                attempts,
                prototype_id,
                min_scale: parameters.f32("min_scale")?,
                max_scale: parameters.f32("max_scale")?,
                random_rotation: parameters.bool("random_rotation")?,
            },
        )
        .map_err(|source| {
            StageBuildError::from_stage("failed to build Poisson Scatter stage", source)
        })?;

        Ok(Box::new(stage))
    }
}
