//! Stage definitions for TerraKit's first-party built-in stages.
//!
//! These definitions expose stable schema metadata and factories for the
//! concrete built-in [`terrakit_pipeline::TerrainStage`] implementations. They
//! are a deterministic catalogue, not a graph and not global mutable state.

mod amplify_height;
mod flat_height;
mod height_field_mesh;
mod noise_height;
mod poisson_scatter;
mod temperature;

pub use amplify_height::AmplifyHeightDefinition;
pub use flat_height::FlatHeightDefinition;
pub use height_field_mesh::HeightFieldMeshDefinition;
pub use noise_height::NoiseHeightDefinition;
pub use poisson_scatter::PoissonScatterDefinition;
pub use temperature::TemperatureMapDefinition;

use terrakit_pipeline::{
    DefinitionError, EnumOption, EnumValueId, InputPortDefinition, OutputPortDefinition,
    ParameterDefinition, ParameterId, ParameterType, ParameterValue, PortId, RegistryError,
    ResourceKind, StageRegistry, StageSchema, StageSchemaVersion, StageTypeId,
};

/// Registers all built-in stage definitions into the supplied registry.
pub fn register_builtin_stage_definitions(
    registry: &mut StageRegistry,
) -> Result<(), RegistryError> {
    registry.register(FlatHeightDefinition::new()?)?;
    registry.register(NoiseHeightDefinition::new()?)?;
    registry.register(HeightFieldMeshDefinition::new()?)?;
    registry.register(PoissonScatterDefinition::new()?)?;
    registry.register(AmplifyHeightDefinition::new()?)?;
    registry.register(TemperatureMapDefinition::new()?)?;

    Ok(())
}

/// Creates a registry containing all built-in stage definitions.
pub fn builtin_stage_registry() -> Result<StageRegistry, RegistryError> {
    let mut registry = StageRegistry::new();
    register_builtin_stage_definitions(&mut registry)?;

    Ok(registry)
}

fn stage_type_id(value: &'static str) -> Result<StageTypeId, DefinitionError> {
    StageTypeId::try_new(value)
}

fn port_id(value: &'static str) -> Result<PortId, DefinitionError> {
    PortId::try_new(value)
}

fn parameter_id(value: &'static str) -> Result<ParameterId, DefinitionError> {
    ParameterId::try_new(value)
}

fn enum_value_id(value: &'static str) -> Result<EnumValueId, DefinitionError> {
    EnumValueId::try_new(value)
}

fn input(
    id: &'static str,
    display_name: &'static str,
    description: &'static str,
    resource_kind: ResourceKind,
) -> Result<InputPortDefinition, DefinitionError> {
    InputPortDefinition::required(port_id(id)?, display_name, description, resource_kind)
}

fn output(
    id: &'static str,
    display_name: &'static str,
    description: &'static str,
    resource_kind: ResourceKind,
) -> Result<OutputPortDefinition, DefinitionError> {
    OutputPortDefinition::new(port_id(id)?, display_name, description, resource_kind)
}

fn parameter(
    id: &'static str,
    display_name: &'static str,
    description: &'static str,
    value_type: ParameterType,
    default: Option<ParameterValue>,
) -> Result<ParameterDefinition, DefinitionError> {
    ParameterDefinition::new(
        parameter_id(id)?,
        display_name,
        description,
        value_type,
        default,
    )
}

fn enum_option(
    id: &'static str,
    display_name: &'static str,
    description: &'static str,
) -> Result<EnumOption, DefinitionError> {
    EnumOption::new(enum_value_id(id)?, display_name, description)
}

// This helper mirrors StageSchema::new while keeping built-in definitions concise.
#[allow(clippy::too_many_arguments)]
fn schema(
    type_id: &'static str,
    version: StageSchemaVersion,
    display_name: &'static str,
    category: &'static str,
    description: &'static str,
    inputs: Vec<InputPortDefinition>,
    outputs: Vec<OutputPortDefinition>,
    parameters: Vec<ParameterDefinition>,
) -> Result<StageSchema, DefinitionError> {
    StageSchema::new(
        stage_type_id(type_id)?,
        version,
        display_name,
        category,
        description,
        inputs,
        outputs,
        parameters,
    )
}
