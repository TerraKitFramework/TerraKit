//! Schema and factory for the built-in noise-height stage.

use terrakit_algorithms::noise::{FractalSettings, MAX_FRACTAL_OCTAVES, NoiseAlgorithm};
use terrakit_core::SeedDomain;
use terrakit_pipeline::{
    DefinitionError, EnumValueId, ParameterType, ParameterValue, ResolvedParameterSet,
    ResourceKind, StageBuildError, StageDefinition, StageId, StageSchema, StageSchemaVersion,
    ValidatedStageBindings,
};

use crate::{HeightNoiseMode, NoiseHeightStage};

use super::{enum_option, enum_value_id, input, output, parameter, schema};

/// Stage definition for applying deterministic fractal noise to a height field.
pub struct NoiseHeightDefinition {
    schema: StageSchema,
}

impl NoiseHeightDefinition {
    /// Stable stage type ID for height noise.
    pub const TYPE_ID: &'static str = "terrakit.height.noise";

    /// Compatible schema revision for this built-in definition.
    pub const SCHEMA_VERSION: StageSchemaVersion = StageSchemaVersion::V1;

    /// Creates the built-in height noise stage definition.
    pub fn new() -> Result<Self, DefinitionError> {
        Ok(Self {
            schema: schema(
                Self::TYPE_ID,
                Self::SCHEMA_VERSION,
                "Noise Height",
                "Height",
                "Reads a source height field and creates a noisy height field.",
                vec![input(
                    "source",
                    "Source",
                    "Source height field to read.",
                    ResourceKind::HeightField,
                )?],
                vec![output(
                    "height",
                    "Height",
                    "Created noisy height field.",
                    ResourceKind::HeightField,
                )?],
                vec![
                    parameter(
                        "algorithm",
                        "Algorithm",
                        "Built-in deterministic noise algorithm.",
                        ParameterType::Enum {
                            options: vec![
                                enum_option("value", "Value", "Coherent value noise.")?,
                                enum_option(
                                    "perlin",
                                    "Perlin",
                                    "Coherent gradient noise using Perlin-style lattice gradients.",
                                )?,
                                enum_option(
                                    "simplex",
                                    "Simplex",
                                    "Coherent gradient noise using a simplex lattice.",
                                )?,
                                enum_option("worley", "Worley", "Signed cellular distance noise.")?,
                            ],
                        },
                        Some(ParameterValue::Enum(enum_value_id("value")?)),
                    )?,
                    parameter(
                        "octaves",
                        "Octaves",
                        "Number of fractal octaves.",
                        ParameterType::U64 {
                            minimum: Some(1),
                            maximum: Some(u64::from(MAX_FRACTAL_OCTAVES)),
                        },
                        Some(ParameterValue::U64(4)),
                    )?,
                    parameter(
                        "frequency",
                        "Frequency",
                        "Initial coordinate multiplier.",
                        ParameterType::F64 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F64(0.01)),
                    )?,
                    parameter(
                        "lacunarity",
                        "Lacunarity",
                        "Per-octave frequency multiplier.",
                        ParameterType::F64 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F64(2.0)),
                    )?,
                    parameter(
                        "persistence",
                        "Persistence",
                        "Per-octave amplitude multiplier.",
                        ParameterType::F32 {
                            minimum: Some(0.0),
                            maximum: None,
                        },
                        Some(ParameterValue::F32(0.5)),
                    )?,
                    parameter(
                        "amplitude",
                        "Amplitude",
                        "Initial noise amplitude.",
                        ParameterType::F32 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F32(1.0)),
                    )?,
                    parameter(
                        "normalize",
                        "Normalize",
                        "Normalize by accumulated absolute octave weight.",
                        ParameterType::Bool,
                        Some(ParameterValue::Bool(true)),
                    )?,
                    parameter(
                        "seed_domain",
                        "Seed Domain",
                        "Domain used to derive this stage's deterministic noise seed.",
                        ParameterType::U64 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::U64(0)),
                    )?,
                    parameter(
                        "mode",
                        "Mode",
                        "How sampled noise is combined with source height.",
                        ParameterType::Enum {
                            options: vec![
                                enum_option("add", "Add", "Add sampled noise to source height.")?,
                                enum_option(
                                    "replace",
                                    "Replace",
                                    "Replace source height with sampled noise.",
                                )?,
                                enum_option(
                                    "multiply",
                                    "Multiply",
                                    "Multiply source height by sampled noise.",
                                )?,
                            ],
                        },
                        Some(ParameterValue::Enum(enum_value_id("add")?)),
                    )?,
                ],
            )?,
        })
    }
}

impl StageDefinition for NoiseHeightDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn terrakit_pipeline::TerrainStage>, StageBuildError> {
        let algorithm = noise_algorithm(parameters.enum_id("algorithm")?)?;
        let octaves = u8::try_from(parameters.u64("octaves")?)
            .map_err(|_| StageBuildError::new("octaves must fit into u8"))?;
        let settings = FractalSettings {
            octaves,
            frequency: parameters.f64("frequency")?,
            lacunarity: parameters.f64("lacunarity")?,
            persistence: parameters.f32("persistence")?,
            amplitude: parameters.f32("amplitude")?,
            normalize: parameters.bool("normalize")?,
        };
        let mode = noise_mode(parameters.enum_id("mode")?)?;
        let stage = NoiseHeightStage::new(
            stage_id,
            bindings.required_input("source")?,
            bindings.output("height")?,
            algorithm,
            settings,
            SeedDomain::new(parameters.u64("seed_domain")?),
            mode,
        )
        .map_err(|source| {
            StageBuildError::from_stage("failed to build Noise Height stage", source)
        })?;

        Ok(Box::new(stage))
    }
}

fn noise_algorithm(value: &EnumValueId) -> Result<NoiseAlgorithm, StageBuildError> {
    match value.as_str() {
        "value" => Ok(NoiseAlgorithm::Value),
        "perlin" => Ok(NoiseAlgorithm::Perlin),
        "simplex" => Ok(NoiseAlgorithm::Simplex),
        "worley" => Ok(NoiseAlgorithm::Worley),
        other => Err(StageBuildError::new(format!(
            "unknown noise algorithm enum id '{other}'"
        ))),
    }
}

fn noise_mode(value: &EnumValueId) -> Result<HeightNoiseMode, StageBuildError> {
    match value.as_str() {
        "add" => Ok(HeightNoiseMode::Add),
        "replace" => Ok(HeightNoiseMode::Replace),
        "multiply" => Ok(HeightNoiseMode::Multiply),
        other => Err(StageBuildError::new(format!(
            "unknown height noise mode enum id '{other}'"
        ))),
    }
}
