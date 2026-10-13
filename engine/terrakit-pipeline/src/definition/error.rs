//! Error types for definition schemas, parameters, bindings, and assembly.

use std::{error::Error, fmt};

use crate::{ResourceKey, ResourceKind, StageError, StageId};

use super::{EnumValueId, ParameterId, ParameterKind, PortId, StageSchemaVersion, StageTypeId};

/// Reason a stable identifier failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierErrorReason {
    /// The identifier contained no bytes.
    Empty,

    /// The identifier contained only whitespace.
    WhitespaceOnly,

    /// The identifier contained a control character.
    ControlCharacter,

    /// The identifier exceeded the configured byte limit.
    TooLong {
        /// Maximum accepted byte length.
        maximum: usize,
    },
}

impl fmt::Display for IdentifierErrorReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("is empty"),
            Self::WhitespaceOnly => formatter.write_str("is whitespace-only"),
            Self::ControlCharacter => formatter.write_str("contains a control character"),
            Self::TooLong { maximum } => {
                write!(formatter, "is longer than {maximum} bytes")
            }
        }
    }
}

/// Numeric value attached to parameter validation diagnostics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericValue {
    /// Signed 64-bit integer value.
    I64(i64),

    /// Unsigned 64-bit integer value.
    U64(u64),

    /// 32-bit floating-point value.
    F32(f32),

    /// 64-bit floating-point value.
    F64(f64),
}

impl fmt::Display for NumericValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I64(value) => write!(formatter, "{value}"),
            Self::U64(value) => write!(formatter, "{value}"),
            Self::F32(value) => write!(formatter, "{value}"),
            Self::F64(value) => write!(formatter, "{value}"),
        }
    }
}

/// Error produced while constructing stage definitions and schema metadata.
#[derive(Debug, Clone, PartialEq)]
pub enum DefinitionError {
    /// A stable identifier was empty, malformed, or too long.
    InvalidIdentifier {
        /// Kind of identifier being validated.
        kind: &'static str,

        /// Rejected identifier text.
        value: String,

        /// Specific validation failure.
        reason: IdentifierErrorReason,
    },

    /// A stage schema version used the reserved zero value.
    InvalidSchemaVersion {
        /// Rejected raw schema version.
        value: u32,
    },

    /// A display label that must be shown to users was empty.
    EmptyDisplayName {
        /// Kind of schema item with the empty label.
        item: &'static str,
    },

    /// A stage schema used an empty category label.
    EmptyCategory {
        /// Stage type whose category was empty.
        stage_type: StageTypeId,
    },

    /// Two input port definitions used the same local port ID.
    DuplicateInputPort {
        /// Duplicated input port ID.
        port: PortId,
    },

    /// Two output port definitions used the same local port ID.
    DuplicateOutputPort {
        /// Duplicated output port ID.
        port: PortId,
    },

    /// Two parameter definitions used the same parameter ID.
    DuplicateParameter {
        /// Duplicated parameter ID.
        parameter: ParameterId,
    },

    /// A parameter default did not match its declared type or constraints.
    InvalidDefault {
        /// Parameter with the invalid default.
        parameter: ParameterId,

        /// Human-readable validation detail.
        message: String,
    },

    /// An enum parameter had empty, duplicate, or inconsistent options.
    InvalidEnumOptions {
        /// Enum parameter with invalid options.
        parameter: ParameterId,

        /// Human-readable validation detail.
        message: String,
    },

    /// A numeric parameter declared invalid bounds.
    InvalidNumericRange {
        /// Numeric parameter with invalid bounds.
        parameter: ParameterId,

        /// Human-readable validation detail.
        message: String,
    },
}

impl DefinitionError {
    pub(crate) fn invalid_identifier(
        kind: &'static str,
        value: &str,
        reason: IdentifierErrorReason,
    ) -> Self {
        Self::InvalidIdentifier {
            kind,
            value: value.to_owned(),
            reason,
        }
    }
}

impl fmt::Display for DefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier {
                kind,
                value,
                reason,
            } => write!(formatter, "invalid {kind} '{value}': {reason}"),
            Self::InvalidSchemaVersion { value } => {
                write!(
                    formatter,
                    "invalid stage schema version {value}: zero is reserved"
                )
            }
            Self::EmptyDisplayName { item } => {
                write!(formatter, "{item} display name must not be empty")
            }
            Self::EmptyCategory { stage_type } => {
                write!(
                    formatter,
                    "stage type '{stage_type}' category must not be empty"
                )
            }
            Self::DuplicateInputPort { port } => {
                write!(formatter, "duplicate input port '{port}'")
            }
            Self::DuplicateOutputPort { port } => {
                write!(formatter, "duplicate output port '{port}'")
            }
            Self::DuplicateParameter { parameter } => {
                write!(formatter, "duplicate parameter '{parameter}'")
            }
            Self::InvalidDefault { parameter, message } => {
                write!(
                    formatter,
                    "invalid default for parameter '{parameter}': {message}"
                )
            }
            Self::InvalidEnumOptions { parameter, message } => {
                write!(
                    formatter,
                    "invalid enum options for parameter '{parameter}': {message}"
                )
            }
            Self::InvalidNumericRange { parameter, message } => {
                write!(
                    formatter,
                    "invalid numeric range for parameter '{parameter}': {message}"
                )
            }
        }
    }
}

impl Error for DefinitionError {}

/// Error produced while resolving caller-supplied parameter values.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterError {
    /// A typed getter received text that is not a valid parameter ID.
    InvalidParameterIdentifier {
        /// Rejected identifier text.
        id: String,
    },

    /// The supplied parameter set contained an ID not declared by the schema.
    UnknownParameter {
        /// Unknown parameter ID.
        parameter: ParameterId,
    },

    /// The parameter has no supplied value and no default.
    MissingRequiredParameter {
        /// Parameter missing a value.
        parameter: ParameterId,
    },

    /// A supplied value used the wrong broad parameter kind.
    WrongValueType {
        /// Parameter with the mismatched value.
        parameter: ParameterId,

        /// Expected parameter kind.
        expected: ParameterKind,

        /// Actual supplied value kind.
        actual: ParameterKind,
    },

    /// A floating-point parameter or vector contained a non-finite value.
    NonFiniteNumericValue {
        /// Parameter with the non-finite value.
        parameter: ParameterId,
    },

    /// A numeric value was lower than its inclusive minimum.
    BelowMinimum {
        /// Parameter below the minimum.
        parameter: ParameterId,

        /// Inclusive minimum.
        minimum: NumericValue,

        /// Supplied value.
        actual: NumericValue,
    },

    /// A numeric value was higher than its inclusive maximum.
    AboveMaximum {
        /// Parameter above the maximum.
        parameter: ParameterId,

        /// Inclusive maximum.
        maximum: NumericValue,

        /// Supplied value.
        actual: NumericValue,
    },

    /// An enum value was not one of the declared options.
    UnknownEnumValue {
        /// Enum parameter being resolved.
        parameter: ParameterId,

        /// Unknown enum value ID.
        value: EnumValueId,
    },
}

impl fmt::Display for ParameterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameterIdentifier { id } => {
                write!(formatter, "invalid parameter identifier '{id}'")
            }
            Self::UnknownParameter { parameter } => {
                write!(formatter, "unknown parameter '{parameter}'")
            }
            Self::MissingRequiredParameter { parameter } => {
                write!(formatter, "missing required parameter '{parameter}'")
            }
            Self::WrongValueType {
                parameter,
                expected,
                actual,
            } => write!(
                formatter,
                "parameter '{parameter}' expected {expected:?}, found {actual:?}"
            ),
            Self::NonFiniteNumericValue { parameter } => {
                write!(
                    formatter,
                    "parameter '{parameter}' contains a non-finite value"
                )
            }
            Self::BelowMinimum {
                parameter,
                minimum,
                actual,
            } => write!(
                formatter,
                "parameter '{parameter}' value {actual} is below minimum {minimum}"
            ),
            Self::AboveMaximum {
                parameter,
                maximum,
                actual,
            } => write!(
                formatter,
                "parameter '{parameter}' value {actual} is above maximum {maximum}"
            ),
            Self::UnknownEnumValue { parameter, value } => write!(
                formatter,
                "parameter '{parameter}' has unknown enum value '{value}'"
            ),
        }
    }
}

impl Error for ParameterError {}

/// Error produced while validating interface-resolved port bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingError {
    /// A typed getter received text that is not a valid port ID.
    InvalidPortIdentifier {
        /// Rejected identifier text.
        id: String,
    },

    /// An input binding referred to a port not declared by the stage schema.
    UnknownInputPort {
        /// Unknown input port ID.
        port: PortId,
    },

    /// An output binding referred to a port not declared by the stage schema.
    UnknownOutputPort {
        /// Unknown output port ID.
        port: PortId,
    },

    /// A required input port did not receive a resource key.
    MissingRequiredInput {
        /// Missing input port ID.
        port: PortId,
    },

    /// An output port did not receive its create-only resource key.
    MissingOutput {
        /// Missing output port ID.
        port: PortId,
    },

    /// The same input port was bound more than once.
    DuplicateInputBinding {
        /// Duplicated input port ID.
        port: PortId,
    },

    /// The same output port was bound more than once.
    DuplicateOutputBinding {
        /// Duplicated output port ID.
        port: PortId,
    },

    /// Two output ports were assigned the same resource key.
    DuplicateOutputResource {
        /// Duplicated output resource key.
        resource: ResourceKey,
    },

    /// A stage attempted to use the same key for an input and an output.
    InputOutputAlias {
        /// Aliased resource key.
        resource: ResourceKey,
    },
}

impl fmt::Display for BindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPortIdentifier { id } => {
                write!(formatter, "invalid port identifier '{id}'")
            }
            Self::UnknownInputPort { port } => write!(formatter, "unknown input port '{port}'"),
            Self::UnknownOutputPort { port } => write!(formatter, "unknown output port '{port}'"),
            Self::MissingRequiredInput { port } => {
                write!(
                    formatter,
                    "missing required input binding for port '{port}'"
                )
            }
            Self::MissingOutput { port } => {
                write!(formatter, "missing output binding for port '{port}'")
            }
            Self::DuplicateInputBinding { port } => {
                write!(formatter, "duplicate input binding for port '{port}'")
            }
            Self::DuplicateOutputBinding { port } => {
                write!(formatter, "duplicate output binding for port '{port}'")
            }
            Self::DuplicateOutputResource { resource } => {
                write!(formatter, "duplicate output resource {resource:?}")
            }
            Self::InputOutputAlias { resource } => {
                write!(
                    formatter,
                    "input and output resource {resource:?} must be distinct"
                )
            }
        }
    }
}

impl Error for BindingError {}

/// Error produced while registering stage definitions.
#[derive(Debug)]
pub enum RegistryError {
    /// A definition with the same stage type ID was already registered.
    DuplicateStageType {
        /// Duplicated stage type ID.
        stage_type: StageTypeId,
    },

    /// A built-in or helper failed while creating schema metadata.
    Definition {
        /// Underlying schema construction error.
        source: DefinitionError,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateStageType { stage_type } => {
                write!(formatter, "duplicate stage type '{stage_type}'")
            }
            Self::Definition { source } => {
                write!(formatter, "stage definition error: {source}")
            }
        }
    }
}

impl Error for RegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Definition { source } => Some(source),
            _ => None,
        }
    }
}

impl From<DefinitionError> for RegistryError {
    fn from(source: DefinitionError) -> Self {
        Self::Definition { source }
    }
}

/// Error produced by a stage definition factory.
#[derive(Debug)]
pub enum StageBuildError {
    /// Contextual factory failure without a nested source.
    Message {
        /// Human-readable failure detail.
        message: String,
    },

    /// A concrete stage constructor rejected the resolved configuration.
    Stage {
        /// Human-readable factory context.
        message: String,

        /// Concrete stage error.
        source: StageError,
    },

    /// A factory parameter getter failed.
    Parameter {
        /// Underlying parameter error.
        source: ParameterError,
    },

    /// A factory binding getter failed.
    Binding {
        /// Underlying binding error.
        source: BindingError,
    },
}

impl StageBuildError {
    /// Creates a contextual factory error without a nested source.
    pub fn new(message: impl Into<String>) -> Self {
        Self::Message {
            message: message.into(),
        }
    }

    /// Creates a contextual factory error wrapping a concrete stage error.
    pub fn from_stage(message: impl Into<String>, source: StageError) -> Self {
        Self::Stage {
            message: message.into(),
            source,
        }
    }
}

impl From<StageError> for StageBuildError {
    fn from(source: StageError) -> Self {
        Self::from_stage("stage constructor rejected resolved configuration", source)
    }
}

impl From<ParameterError> for StageBuildError {
    fn from(source: ParameterError) -> Self {
        Self::Parameter { source }
    }
}

impl From<BindingError> for StageBuildError {
    fn from(source: BindingError) -> Self {
        Self::Binding { source }
    }
}

impl fmt::Display for StageBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message { message } => formatter.write_str(message),
            Self::Stage { message, source } => write!(formatter, "{message}: {source}"),
            Self::Parameter { source } => write!(formatter, "parameter lookup failed: {source}"),
            Self::Binding { source } => write!(formatter, "binding lookup failed: {source}"),
        }
    }
}

impl Error for StageBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Stage { source, .. } => Some(source),
            Self::Parameter { source } => Some(source),
            Self::Binding { source } => Some(source),
            Self::Message { .. } => None,
        }
    }
}

/// Error produced while assembling an executable terrain pipeline.
#[derive(Debug)]
pub enum PipelineAssemblyError {
    /// Two construction requests used the same executable stage instance ID.
    DuplicateStageId {
        /// Duplicated stage instance ID.
        stage_id: StageId,
    },

    /// No registered definition matched the requested stage type.
    UnknownStageType {
        /// Unknown stage type ID.
        stage_type: StageTypeId,
    },

    /// The construction request targeted a different schema version than the registered definition.
    SchemaVersionMismatch {
        /// Stage being assembled.
        stage_id: StageId,

        /// Stage type whose schema version was checked.
        stage_type: StageTypeId,

        /// Schema version supplied by the caller.
        requested: StageSchemaVersion,

        /// Schema version registered for the stage type.
        available: StageSchemaVersion,
    },

    /// Parameter resolution failed for a stage.
    Parameter {
        /// Stage being assembled.
        stage_id: StageId,

        /// Underlying parameter error.
        source: ParameterError,
    },

    /// Binding validation failed for a stage.
    Binding {
        /// Stage being assembled.
        stage_id: StageId,

        /// Underlying binding error.
        source: BindingError,
    },

    /// An input key had not been produced by an earlier stage.
    MissingInputResource {
        /// Stage being assembled.
        stage_id: StageId,

        /// Input port requiring the missing key.
        port: PortId,

        /// Missing resource key.
        key: ResourceKey,
    },

    /// A produced resource had the wrong kind for an input port.
    WrongInputResourceKind {
        /// Stage being assembled.
        stage_id: StageId,

        /// Input port receiving the key.
        port: PortId,

        /// Resource key being checked.
        key: ResourceKey,

        /// Kind declared by the input port.
        expected: ResourceKind,

        /// Kind previously produced for the key.
        actual: ResourceKind,
    },

    /// An output key was already produced by an earlier stage.
    OutputResourceAlreadyAssigned {
        /// Stage being assembled.
        stage_id: StageId,

        /// Output port assigning the duplicate key.
        port: PortId,

        /// Duplicate resource key.
        key: ResourceKey,
    },

    /// A stage attempted to reuse one key as both input and output.
    InputOutputAlias {
        /// Stage being assembled.
        stage_id: StageId,

        /// Aliased resource key.
        key: ResourceKey,
    },

    /// The stage definition factory failed.
    StageBuild {
        /// Stage being assembled.
        stage_id: StageId,

        /// Stage type whose factory failed.
        stage_type: StageTypeId,

        /// Underlying factory error.
        source: StageBuildError,
    },

    /// A factory returned a stage that did not match its declared schema.
    StageContractMismatch {
        /// Stage being assembled.
        stage_id: StageId,

        /// Human-readable mismatch detail.
        message: String,
    },
}

impl fmt::Display for PipelineAssemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateStageId { stage_id } => {
                write!(formatter, "duplicate stage id {stage_id:?}")
            }
            Self::UnknownStageType { stage_type } => {
                write!(formatter, "unknown stage type '{stage_type}'")
            }
            Self::SchemaVersionMismatch {
                stage_id,
                stage_type,
                requested,
                available,
            } => write!(
                formatter,
                "stage {} requested schema version {} for '{stage_type}', but version {} is registered",
                stage_id.0, requested, available
            ),
            Self::Parameter { stage_id, source } => {
                write!(formatter, "stage {stage_id:?} parameter error: {source}")
            }
            Self::Binding { stage_id, source } => {
                write!(formatter, "stage {stage_id:?} binding error: {source}")
            }
            Self::MissingInputResource {
                stage_id,
                port,
                key,
            } => write!(
                formatter,
                "stage {stage_id:?} input port '{port}' requires {key:?}, but no earlier stage produced it"
            ),
            Self::WrongInputResourceKind {
                stage_id,
                port,
                key,
                expected,
                actual,
            } => write!(
                formatter,
                "stage {stage_id:?} input port '{port}' expected {key:?} to be {expected:?}, found {actual:?}"
            ),
            Self::OutputResourceAlreadyAssigned {
                stage_id,
                port,
                key,
            } => write!(
                formatter,
                "stage {stage_id:?} output port '{port}' cannot assign already-produced resource {key:?}"
            ),
            Self::InputOutputAlias { stage_id, key } => write!(
                formatter,
                "stage {stage_id:?} cannot use {key:?} as both input and output"
            ),
            Self::StageBuild {
                stage_id,
                stage_type,
                source,
            } => write!(
                formatter,
                "stage {stage_id:?} of type '{stage_type}' failed to build: {source}"
            ),
            Self::StageContractMismatch { stage_id, message } => {
                write!(formatter, "stage {stage_id:?} contract mismatch: {message}")
            }
        }
    }
}

impl Error for PipelineAssemblyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parameter { source, .. } => Some(source),
            Self::Binding { source, .. } => Some(source),
            Self::StageBuild { source, .. } => Some(source),
            _ => None,
        }
    }
}
