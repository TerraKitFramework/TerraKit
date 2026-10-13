//! Temperature map stage definition.

use terrakit_core::SeedDomain;

use terrakit_pipeline::{
    DefinitionError, ParameterType, ParameterValue, ResolvedParameterSet, ResourceKind,
    StageBuildError, StageDefinition, StageId, StageSchema, StageSchemaVersion,
    ValidatedStageBindings,
};

use crate::height::TemperatureMapStage;

use super::{output, parameter, schema};

/// Stage definition for generating a noise-based temperature map as a HeightField resource.
pub struct TemperatureMapDefinition {
    schema: StageSchema,
}

impl TemperatureMapDefinition {
    /// Stable stage type ID for temperature map generation.
    pub const TYPE_ID: &'static str = "terrakit.height.temperature";
    /// Compatible schema revision for this built-in definition.
    pub const SCHEMA_VERSION: StageSchemaVersion = StageSchemaVersion::V1;

    /// Creates the built-in temperature map stage definition.
    pub fn new() -> Result<Self, DefinitionError> {
        Ok(Self {
            schema: schema(
                Self::TYPE_ID,
                Self::SCHEMA_VERSION,
                "Temperature Map",
                "Height",
                "Generates a noise-based temperature map as a HeightField resource.",
                Vec::new(),
                vec![output(
                    "height",
                    "Height",
                    "Temperature map as a height field.",
                    ResourceKind::HeightField,
                )?],
                vec![
                    parameter(
                        "scale",
                        "Scale",
                        "Noise sampling scale.",
                        ParameterType::F64 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F64(0.01)),
                    )?,
                    parameter(
                        "seed_domain",
                        "Seed Domain",
                        "Domain used to derive the deterministic temperature noise seed.",
                        ParameterType::U64 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::U64(1)),
                    )?,
                ],
            )?,
        })
    }
}

impl StageDefinition for TemperatureMapDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn terrakit_pipeline::TerrainStage>, StageBuildError> {
        let stage = TemperatureMapStage::new(
            stage_id,
            bindings.output("height")?,
            parameters.f64("scale")?,
            SeedDomain::new(parameters.u64("seed_domain")?),
        )
        .map_err(|source| {
            StageBuildError::from_stage("failed to build Temperature Map stage", source)
        })?;

        Ok(Box::new(stage))
    }
}
