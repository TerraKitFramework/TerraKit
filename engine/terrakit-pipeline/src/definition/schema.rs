//! Discoverable stage schema metadata.

use std::collections::HashSet;

use super::{
    DefinitionError, InputPortDefinition, OutputPortDefinition, ParameterDefinition,
    ParameterError, ParameterId, ParameterSet, PortId, ResolvedParameterSet, StageSchemaVersion,
    StageTypeId,
};

/// Discoverable metadata, ports, parameters, and schema version for one stage type.
///
/// Inputs, outputs, parameters, and enum options retain declaration order for
/// interface presentation. Machine identity comes from stable IDs plus the
/// stage schema version, not from display labels or presentation order.
#[derive(Debug, Clone, PartialEq)]
pub struct StageSchema {
    type_id: StageTypeId,
    version: StageSchemaVersion,
    display_name: Box<str>,
    category: Box<str>,
    description: Box<str>,
    inputs: Vec<InputPortDefinition>,
    outputs: Vec<OutputPortDefinition>,
    parameters: Vec<ParameterDefinition>,
}

impl StageSchema {
    /// Creates a validated stage schema.
    // This constructor mirrors the schema contract as separate stable fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        type_id: StageTypeId,
        version: StageSchemaVersion,
        display_name: impl Into<Box<str>>,
        category: impl Into<Box<str>>,
        description: impl Into<Box<str>>,
        inputs: Vec<InputPortDefinition>,
        outputs: Vec<OutputPortDefinition>,
        parameters: Vec<ParameterDefinition>,
    ) -> Result<Self, DefinitionError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(DefinitionError::EmptyDisplayName { item: "stage" });
        }

        let category = category.into();
        if category.trim().is_empty() {
            return Err(DefinitionError::EmptyCategory {
                stage_type: type_id,
            });
        }

        validate_duplicate_inputs(&inputs)?;
        validate_duplicate_outputs(&outputs)?;
        validate_duplicate_parameters(&parameters)?;

        Ok(Self {
            type_id,
            version,
            display_name,
            category,
            description: description.into(),
            inputs,
            outputs,
            parameters,
        })
    }

    /// Returns the stable stage type ID.
    pub fn type_id(&self) -> &StageTypeId {
        &self.type_id
    }

    /// Returns this stage type's schema version.
    pub const fn version(&self) -> StageSchemaVersion {
        self.version
    }

    /// Returns the user-facing stage name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the user-facing category label.
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Returns the user-facing stage description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns input port definitions in interface display order.
    pub fn inputs(&self) -> &[InputPortDefinition] {
        &self.inputs
    }

    /// Returns output port definitions in interface display order.
    pub fn outputs(&self) -> &[OutputPortDefinition] {
        &self.outputs
    }

    /// Returns parameter definitions in interface display order.
    pub fn parameters(&self) -> &[ParameterDefinition] {
        &self.parameters
    }

    /// Looks up an input port by stable ID.
    pub fn input(&self, id: &PortId) -> Option<&InputPortDefinition> {
        self.inputs.iter().find(|input| input.id() == id)
    }

    /// Looks up an output port by stable ID.
    pub fn output(&self, id: &PortId) -> Option<&OutputPortDefinition> {
        self.outputs.iter().find(|output| output.id() == id)
    }

    /// Looks up a parameter by stable ID.
    pub fn parameter(&self, id: &ParameterId) -> Option<&ParameterDefinition> {
        self.parameters
            .iter()
            .find(|parameter| parameter.id() == id)
    }

    /// Resolves caller-supplied parameter values against this schema.
    pub fn resolve_parameters(
        &self,
        supplied: &ParameterSet,
    ) -> Result<ResolvedParameterSet, ParameterError> {
        ResolvedParameterSet::resolve(&self.parameters, supplied)
    }
}

fn validate_duplicate_inputs(inputs: &[InputPortDefinition]) -> Result<(), DefinitionError> {
    let mut ids = HashSet::new();
    for input in inputs {
        if !ids.insert(input.id()) {
            return Err(DefinitionError::DuplicateInputPort {
                port: input.id().clone(),
            });
        }
    }

    Ok(())
}

fn validate_duplicate_outputs(outputs: &[OutputPortDefinition]) -> Result<(), DefinitionError> {
    let mut ids = HashSet::new();
    for output in outputs {
        if !ids.insert(output.id()) {
            return Err(DefinitionError::DuplicateOutputPort {
                port: output.id().clone(),
            });
        }
    }

    Ok(())
}

fn validate_duplicate_parameters(
    parameters: &[ParameterDefinition],
) -> Result<(), DefinitionError> {
    let mut ids = HashSet::new();
    for parameter in parameters {
        if !ids.insert(parameter.id()) {
            return Err(DefinitionError::DuplicateParameter {
                parameter: parameter.id().clone(),
            });
        }
    }

    Ok(())
}
