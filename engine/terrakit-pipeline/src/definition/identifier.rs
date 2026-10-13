//! Stable identifier newtypes used by stage definitions.

use std::fmt;

use super::{DefinitionError, IdentifierErrorReason};

const STAGE_TYPE_ID_MAX_BYTES: usize = 128;
const LOCAL_ID_MAX_BYTES: usize = 64;

/// Stable machine-readable identifier for a stage type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StageTypeId(Box<str>);

impl StageTypeId {
    /// Creates a validated stage type identifier.
    pub fn try_new(value: impl Into<Box<str>>) -> Result<Self, DefinitionError> {
        let value = value.into();
        validate_identifier("StageTypeId", &value, STAGE_TYPE_ID_MAX_BYTES)?;

        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for StageTypeId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for StageTypeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Identifies one compatible revision of a stage's discoverable schema.
///
/// This version covers the stage's ports, parameters, defaults, constraints,
/// and construction semantics. It is separate from the TerraKit crate version
/// and the future C ABI version.
///
/// Saved stage configuration compatibility is identified by stage type ID plus
/// schema version. Increment the version when existing saved node data could be
/// interpreted differently, such as after stable port or parameter ID changes,
/// resource-kind changes, enum option ID changes, default changes, constraint
/// changes, optional-input changes, or meaningful construction-semantics
/// changes. Presentation-only changes such as spelling fixes or documentation
/// updates do not necessarily require an increment.
///
/// Schema versioning does not yet guarantee byte-for-byte identical generated
/// terrain across all future TerraKit releases.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StageSchemaVersion(u32);

impl StageSchemaVersion {
    /// Initial schema version used by first-party stages.
    pub const V1: Self = Self(1);

    /// Creates a nonzero schema version.
    pub const fn try_new(value: u32) -> Result<Self, DefinitionError> {
        if value == 0 {
            return Err(DefinitionError::InvalidSchemaVersion { value });
        }

        Ok(Self(value))
    }

    /// Returns the raw schema version.
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for StageSchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.value())
    }
}

/// Stable machine-readable identifier for an input or output port.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PortId(Box<str>);

impl PortId {
    /// Creates a validated port identifier.
    pub fn try_new(value: impl Into<Box<str>>) -> Result<Self, DefinitionError> {
        let value = value.into();
        validate_identifier("PortId", &value, LOCAL_ID_MAX_BYTES)?;

        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PortId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for PortId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable machine-readable identifier for an editable parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterId(Box<str>);

impl ParameterId {
    /// Creates a validated parameter identifier.
    pub fn try_new(value: impl Into<Box<str>>) -> Result<Self, DefinitionError> {
        let value = value.into();
        validate_identifier("ParameterId", &value, LOCAL_ID_MAX_BYTES)?;

        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ParameterId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ParameterId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable machine-readable identifier for one enum parameter option.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumValueId(Box<str>);

impl EnumValueId {
    /// Creates a validated enum value identifier.
    pub fn try_new(value: impl Into<Box<str>>) -> Result<Self, DefinitionError> {
        let value = value.into();
        validate_identifier("EnumValueId", &value, LOCAL_ID_MAX_BYTES)?;

        Ok(Self(value))
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for EnumValueId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for EnumValueId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn validate_identifier(
    kind: &'static str,
    value: &str,
    maximum_bytes: usize,
) -> Result<(), DefinitionError> {
    if value.is_empty() {
        return Err(DefinitionError::invalid_identifier(
            kind,
            value,
            IdentifierErrorReason::Empty,
        ));
    }

    if value.trim().is_empty() {
        return Err(DefinitionError::invalid_identifier(
            kind,
            value,
            IdentifierErrorReason::WhitespaceOnly,
        ));
    }

    if value.chars().any(char::is_control) {
        return Err(DefinitionError::invalid_identifier(
            kind,
            value,
            IdentifierErrorReason::ControlCharacter,
        ));
    }

    if value.len() > maximum_bytes {
        return Err(DefinitionError::invalid_identifier(
            kind,
            value,
            IdentifierErrorReason::TooLong {
                maximum: maximum_bytes,
            },
        ));
    }

    Ok(())
}
