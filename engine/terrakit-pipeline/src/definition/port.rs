//! Resource port schema definitions and resolved port-resource bindings.

use std::collections::{HashMap, HashSet};

use crate::{ResourceKey, ResourceKind};

use super::{BindingError, DefinitionError, PortId, StageSchema};

/// Discoverable schema for one input resource port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPortDefinition {
    id: PortId,
    display_name: Box<str>,
    description: Box<str>,
    resource_kind: ResourceKind,
    optional: bool,
}

impl InputPortDefinition {
    /// Creates a validated input port definition.
    pub fn new(
        id: PortId,
        display_name: impl Into<Box<str>>,
        description: impl Into<Box<str>>,
        resource_kind: ResourceKind,
        optional: bool,
    ) -> Result<Self, DefinitionError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(DefinitionError::EmptyDisplayName { item: "input port" });
        }

        Ok(Self {
            id,
            display_name,
            description: description.into(),
            resource_kind,
            optional,
        })
    }

    /// Creates a required input port definition.
    pub fn required(
        id: PortId,
        display_name: impl Into<Box<str>>,
        description: impl Into<Box<str>>,
        resource_kind: ResourceKind,
    ) -> Result<Self, DefinitionError> {
        Self::new(id, display_name, description, resource_kind, false)
    }

    /// Creates an optional input port definition.
    pub fn optional_input(
        id: PortId,
        display_name: impl Into<Box<str>>,
        description: impl Into<Box<str>>,
        resource_kind: ResourceKind,
    ) -> Result<Self, DefinitionError> {
        Self::new(id, display_name, description, resource_kind, true)
    }

    /// Returns the stable port ID.
    pub fn id(&self) -> &PortId {
        &self.id
    }

    /// Returns the user-facing port label.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the user-facing port description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the canonical resource kind accepted by this input.
    pub const fn resource_kind(&self) -> ResourceKind {
        self.resource_kind
    }

    /// Returns true when the input binding may be absent.
    pub const fn optional(&self) -> bool {
        self.optional
    }
}

/// Discoverable schema for one output resource port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPortDefinition {
    id: PortId,
    display_name: Box<str>,
    description: Box<str>,
    resource_kind: ResourceKind,
}

impl OutputPortDefinition {
    /// Creates a validated output port definition.
    pub fn new(
        id: PortId,
        display_name: impl Into<Box<str>>,
        description: impl Into<Box<str>>,
        resource_kind: ResourceKind,
    ) -> Result<Self, DefinitionError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(DefinitionError::EmptyDisplayName {
                item: "output port",
            });
        }

        Ok(Self {
            id,
            display_name,
            description: description.into(),
            resource_kind,
        })
    }

    /// Returns the stable port ID.
    pub fn id(&self) -> &PortId {
        &self.id
    }

    /// Returns the user-facing port label.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the user-facing port description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the canonical resource kind created by this output.
    pub const fn resource_kind(&self) -> ResourceKind {
        self.resource_kind
    }
}

/// Resolved binding from a local stage port to a pipeline resource key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortBinding {
    port: PortId,
    resource: ResourceKey,
}

impl PortBinding {
    /// Creates a port-resource binding.
    pub const fn new(port: PortId, resource: ResourceKey) -> Self {
        Self { port, resource }
    }

    /// Returns the local port ID.
    pub fn port(&self) -> &PortId {
        &self.port
    }

    /// Returns the resource key assigned to the port.
    pub const fn resource(&self) -> ResourceKey {
        self.resource
    }
}

/// Resolved input and output bindings for one stage construction request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageBindings {
    inputs: Vec<PortBinding>,
    outputs: Vec<PortBinding>,
}

impl StageBindings {
    /// Creates an empty binding set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one input binding, rejecting duplicate input ports.
    pub fn bind_input(&mut self, port: PortId, resource: ResourceKey) -> Result<(), BindingError> {
        if self.inputs.iter().any(|binding| binding.port() == &port) {
            return Err(BindingError::DuplicateInputBinding { port });
        }

        self.inputs.push(PortBinding::new(port, resource));
        Ok(())
    }

    /// Adds one output binding, rejecting duplicate output ports.
    pub fn bind_output(&mut self, port: PortId, resource: ResourceKey) -> Result<(), BindingError> {
        if self.outputs.iter().any(|binding| binding.port() == &port) {
            return Err(BindingError::DuplicateOutputBinding { port });
        }

        self.outputs.push(PortBinding::new(port, resource));
        Ok(())
    }

    /// Returns a new binding set with an additional input binding.
    pub fn with_input(mut self, port: PortId, resource: ResourceKey) -> Result<Self, BindingError> {
        self.bind_input(port, resource)?;
        Ok(self)
    }

    /// Returns a new binding set with an additional output binding.
    pub fn with_output(
        mut self,
        port: PortId,
        resource: ResourceKey,
    ) -> Result<Self, BindingError> {
        self.bind_output(port, resource)?;
        Ok(self)
    }

    /// Returns input bindings in caller-supplied order.
    pub fn inputs(&self) -> &[PortBinding] {
        &self.inputs
    }

    /// Returns output bindings in caller-supplied order.
    pub fn outputs(&self) -> &[PortBinding] {
        &self.outputs
    }
}

/// Schema-validated bindings for one stage construction request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidatedStageBindings {
    inputs: HashMap<PortId, ResourceKey>,
    outputs: HashMap<PortId, ResourceKey>,
}

impl ValidatedStageBindings {
    /// Validates raw bindings against a stage schema.
    pub fn validate(schema: &StageSchema, bindings: &StageBindings) -> Result<Self, BindingError> {
        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();
        let mut output_resources = HashSet::new();

        for binding in bindings.inputs() {
            let port = binding.port();
            if schema.input(port).is_none() {
                return Err(BindingError::UnknownInputPort { port: port.clone() });
            }

            if inputs.insert(port.clone(), binding.resource()).is_some() {
                return Err(BindingError::DuplicateInputBinding { port: port.clone() });
            }
        }

        for binding in bindings.outputs() {
            let port = binding.port();
            if schema.output(port).is_none() {
                return Err(BindingError::UnknownOutputPort { port: port.clone() });
            }

            if outputs.insert(port.clone(), binding.resource()).is_some() {
                return Err(BindingError::DuplicateOutputBinding { port: port.clone() });
            }

            if !output_resources.insert(binding.resource()) {
                return Err(BindingError::DuplicateOutputResource {
                    resource: binding.resource(),
                });
            }
        }

        for input in schema.inputs() {
            if !input.optional() && !inputs.contains_key(input.id()) {
                return Err(BindingError::MissingRequiredInput {
                    port: input.id().clone(),
                });
            }
        }

        for output in schema.outputs() {
            if !outputs.contains_key(output.id()) {
                return Err(BindingError::MissingOutput {
                    port: output.id().clone(),
                });
            }
        }

        let input_resources: HashSet<ResourceKey> = inputs.values().copied().collect();
        for resource in outputs.values() {
            if input_resources.contains(resource) {
                return Err(BindingError::InputOutputAlias {
                    resource: *resource,
                });
            }
        }

        Ok(Self { inputs, outputs })
    }

    /// Returns an optional input resource binding by port ID text.
    pub fn input(&self, id: &str) -> Option<ResourceKey> {
        let port = PortId::try_new(id).ok()?;
        self.inputs.get(&port).copied()
    }

    /// Returns a required input binding by port ID text.
    pub fn required_input(&self, id: &str) -> Result<ResourceKey, BindingError> {
        let port = PortId::try_new(id)
            .map_err(|_| BindingError::InvalidPortIdentifier { id: id.to_owned() })?;
        self.inputs
            .get(&port)
            .copied()
            .ok_or(BindingError::MissingRequiredInput { port })
    }

    /// Returns an output binding by port ID text.
    pub fn output(&self, id: &str) -> Result<ResourceKey, BindingError> {
        let port = PortId::try_new(id)
            .map_err(|_| BindingError::InvalidPortIdentifier { id: id.to_owned() })?;
        self.outputs
            .get(&port)
            .copied()
            .ok_or(BindingError::MissingOutput { port })
    }

    /// Returns validated input bindings in unspecified map order.
    pub fn inputs(&self) -> impl Iterator<Item = (&PortId, &ResourceKey)> {
        self.inputs.iter()
    }

    /// Returns validated output bindings in unspecified map order.
    pub fn outputs(&self) -> impl Iterator<Item = (&PortId, &ResourceKey)> {
        self.outputs.iter()
    }
}
