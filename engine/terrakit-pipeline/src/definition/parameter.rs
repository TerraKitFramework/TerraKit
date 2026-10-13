//! Parameter schema, supplied values, defaults, and resolved parameter sets.

use std::collections::{HashMap, HashSet};

use terrakit_core::{Vector2F64, Vector3F64};

use super::{DefinitionError, EnumValueId, NumericValue, ParameterError, ParameterId};

/// Broad runtime kind of a parameter value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterKind {
    /// Boolean value.
    Bool,

    /// Signed 64-bit integer value.
    I64,

    /// Unsigned 64-bit integer value.
    U64,

    /// 32-bit floating-point value.
    F32,

    /// 64-bit floating-point value.
    F64,

    /// Two-component 64-bit floating-point vector.
    Vector2F64,

    /// Three-component 64-bit floating-point vector.
    Vector3F64,

    /// Owned UTF-8 string.
    String,

    /// Stable enum option ID.
    Enum,
}

/// Caller-supplied or default parameter value.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// Boolean parameter value.
    Bool(bool),

    /// Signed 64-bit integer parameter value.
    I64(i64),

    /// Unsigned 64-bit integer parameter value.
    U64(u64),

    /// 32-bit floating-point parameter value.
    F32(f32),

    /// 64-bit floating-point parameter value.
    F64(f64),

    /// Two-component world-space vector parameter value.
    Vector2F64(Vector2F64),

    /// Three-component world-space vector parameter value.
    Vector3F64(Vector3F64),

    /// Owned string parameter value.
    String(Box<str>),

    /// Stable enum option parameter value.
    Enum(EnumValueId),
}

impl ParameterValue {
    /// Returns the broad kind represented by this value.
    pub const fn kind(&self) -> ParameterKind {
        match self {
            Self::Bool(_) => ParameterKind::Bool,
            Self::I64(_) => ParameterKind::I64,
            Self::U64(_) => ParameterKind::U64,
            Self::F32(_) => ParameterKind::F32,
            Self::F64(_) => ParameterKind::F64,
            Self::Vector2F64(_) => ParameterKind::Vector2F64,
            Self::Vector3F64(_) => ParameterKind::Vector3F64,
            Self::String(_) => ParameterKind::String,
            Self::Enum(_) => ParameterKind::Enum,
        }
    }
}

/// Schema type and constraints accepted by a parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
    /// Boolean value.
    Bool,

    /// Signed 64-bit integer with optional inclusive bounds.
    I64 {
        /// Inclusive minimum accepted value.
        minimum: Option<i64>,

        /// Inclusive maximum accepted value.
        maximum: Option<i64>,
    },

    /// Unsigned 64-bit integer with optional inclusive bounds.
    U64 {
        /// Inclusive minimum accepted value.
        minimum: Option<u64>,

        /// Inclusive maximum accepted value.
        maximum: Option<u64>,
    },

    /// 32-bit floating-point value with optional inclusive bounds.
    F32 {
        /// Inclusive minimum accepted value.
        minimum: Option<f32>,

        /// Inclusive maximum accepted value.
        maximum: Option<f32>,
    },

    /// 64-bit floating-point value with optional inclusive bounds.
    F64 {
        /// Inclusive minimum accepted value.
        minimum: Option<f64>,

        /// Inclusive maximum accepted value.
        maximum: Option<f64>,
    },

    /// Two-component 64-bit floating-point vector.
    Vector2F64,

    /// Three-component 64-bit floating-point vector.
    Vector3F64,

    /// Owned UTF-8 string.
    String,

    /// Enum with a fixed list of stable option IDs.
    Enum {
        /// Available enum options in interface display order.
        options: Vec<EnumOption>,
    },
}

impl ParameterType {
    /// Returns the broad value kind accepted by this type.
    pub const fn kind(&self) -> ParameterKind {
        match self {
            Self::Bool => ParameterKind::Bool,
            Self::I64 { .. } => ParameterKind::I64,
            Self::U64 { .. } => ParameterKind::U64,
            Self::F32 { .. } => ParameterKind::F32,
            Self::F64 { .. } => ParameterKind::F64,
            Self::Vector2F64 => ParameterKind::Vector2F64,
            Self::Vector3F64 => ParameterKind::Vector3F64,
            Self::String => ParameterKind::String,
            Self::Enum { .. } => ParameterKind::Enum,
        }
    }
}

/// One selectable option for an enum parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumOption {
    id: EnumValueId,
    display_name: Box<str>,
    description: Box<str>,
}

impl EnumOption {
    /// Creates an enum option with stable ID and display metadata.
    pub fn new(
        id: EnumValueId,
        display_name: impl Into<Box<str>>,
        description: impl Into<Box<str>>,
    ) -> Result<Self, DefinitionError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(DefinitionError::EmptyDisplayName {
                item: "enum option",
            });
        }

        Ok(Self {
            id,
            display_name,
            description: description.into(),
        })
    }

    /// Returns the stable enum value ID.
    pub fn id(&self) -> &EnumValueId {
        &self.id
    }

    /// Returns the user-facing option label.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the user-facing option description.
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Discoverable schema and default for one editable stage parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDefinition {
    id: ParameterId,
    display_name: Box<str>,
    description: Box<str>,
    value_type: ParameterType,
    default: Option<ParameterValue>,
}

impl ParameterDefinition {
    /// Creates a validated parameter definition.
    pub fn new(
        id: ParameterId,
        display_name: impl Into<Box<str>>,
        description: impl Into<Box<str>>,
        value_type: ParameterType,
        default: Option<ParameterValue>,
    ) -> Result<Self, DefinitionError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(DefinitionError::EmptyDisplayName { item: "parameter" });
        }

        validate_parameter_type(&id, &value_type)?;
        if let Some(default_value) = &default {
            validate_default(&id, &value_type, default_value)?;
        }

        Ok(Self {
            id,
            display_name,
            description: description.into(),
            value_type,
            default,
        })
    }

    /// Returns the stable parameter ID.
    pub fn id(&self) -> &ParameterId {
        &self.id
    }

    /// Returns the user-facing parameter label.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the user-facing parameter description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the declared parameter type and constraints.
    pub fn value_type(&self) -> &ParameterType {
        &self.value_type
    }

    /// Returns the default value, if one exists.
    pub fn default(&self) -> Option<&ParameterValue> {
        self.default.as_ref()
    }

    pub(crate) fn resolve_value(
        &self,
        supplied: Option<&ParameterValue>,
    ) -> Result<ParameterValue, ParameterError> {
        let value =
            match supplied {
                Some(value) => value.clone(),
                None => self.default.clone().ok_or_else(|| {
                    ParameterError::MissingRequiredParameter {
                        parameter: self.id.clone(),
                    }
                })?,
            };

        validate_parameter_value(self, &value)?;

        Ok(value)
    }
}

/// Caller-supplied parameter values keyed by stable parameter ID.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParameterSet {
    values: HashMap<ParameterId, ParameterValue>,
}

impl ParameterSet {
    /// Creates an empty parameter set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces one parameter value.
    pub fn set(&mut self, id: ParameterId, value: ParameterValue) -> Option<ParameterValue> {
        self.values.insert(id, value)
    }

    /// Returns a supplied parameter value by ID.
    pub fn get(&self, id: &ParameterId) -> Option<&ParameterValue> {
        self.values.get(id)
    }

    /// Returns true when a parameter value was supplied.
    pub fn contains(&self, id: &ParameterId) -> bool {
        self.values.contains_key(id)
    }

    /// Returns supplied values in unspecified map order.
    pub fn values(&self) -> impl Iterator<Item = (&ParameterId, &ParameterValue)> {
        self.values.iter()
    }
}

/// Complete validated parameter values resolved against one stage schema.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedParameterSet {
    values: HashMap<ParameterId, ParameterValue>,
}

impl ResolvedParameterSet {
    pub(crate) fn resolve(
        definitions: &[ParameterDefinition],
        supplied: &ParameterSet,
    ) -> Result<Self, ParameterError> {
        for (parameter, _) in supplied.values() {
            if !definitions
                .iter()
                .any(|definition| definition.id() == parameter)
            {
                return Err(ParameterError::UnknownParameter {
                    parameter: parameter.clone(),
                });
            }
        }

        let mut values = HashMap::new();
        for definition in definitions {
            let value = definition.resolve_value(supplied.get(definition.id()))?;
            values.insert(definition.id().clone(), value);
        }

        Ok(Self { values })
    }

    /// Returns a resolved parameter value by ID.
    pub fn get(&self, id: &ParameterId) -> Option<&ParameterValue> {
        self.values.get(id)
    }

    /// Returns resolved values in unspecified map order.
    pub fn values(&self) -> impl Iterator<Item = (&ParameterId, &ParameterValue)> {
        self.values.iter()
    }

    /// Returns a resolved boolean value by stable parameter ID text.
    pub fn bool(&self, id: &str) -> Result<bool, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::Bool(value) => Ok(*value),
            other => Err(wrong_value_type(parameter, ParameterKind::Bool, other)),
        }
    }

    /// Returns a resolved signed integer value by stable parameter ID text.
    pub fn i64(&self, id: &str) -> Result<i64, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::I64(value) => Ok(*value),
            other => Err(wrong_value_type(parameter, ParameterKind::I64, other)),
        }
    }

    /// Returns a resolved unsigned integer value by stable parameter ID text.
    pub fn u64(&self, id: &str) -> Result<u64, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::U64(value) => Ok(*value),
            other => Err(wrong_value_type(parameter, ParameterKind::U64, other)),
        }
    }

    /// Returns a resolved 32-bit floating-point value by stable parameter ID text.
    pub fn f32(&self, id: &str) -> Result<f32, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::F32(value) => Ok(*value),
            other => Err(wrong_value_type(parameter, ParameterKind::F32, other)),
        }
    }

    /// Returns a resolved 64-bit floating-point value by stable parameter ID text.
    pub fn f64(&self, id: &str) -> Result<f64, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::F64(value) => Ok(*value),
            other => Err(wrong_value_type(parameter, ParameterKind::F64, other)),
        }
    }

    /// Returns a resolved two-component vector value by stable parameter ID text.
    pub fn vector2_f64(&self, id: &str) -> Result<Vector2F64, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::Vector2F64(value) => Ok(*value),
            other => Err(wrong_value_type(
                parameter,
                ParameterKind::Vector2F64,
                other,
            )),
        }
    }

    /// Returns a resolved three-component vector value by stable parameter ID text.
    pub fn vector3_f64(&self, id: &str) -> Result<Vector3F64, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::Vector3F64(value) => Ok(*value),
            other => Err(wrong_value_type(
                parameter,
                ParameterKind::Vector3F64,
                other,
            )),
        }
    }

    /// Returns a resolved string value by stable parameter ID text.
    pub fn string(&self, id: &str) -> Result<&str, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::String(value) => Ok(value),
            other => Err(wrong_value_type(parameter, ParameterKind::String, other)),
        }
    }

    /// Returns a resolved enum value ID by stable parameter ID text.
    pub fn enum_id(&self, id: &str) -> Result<&EnumValueId, ParameterError> {
        let (parameter, value) = self.lookup(id)?;
        match value {
            ParameterValue::Enum(value) => Ok(value),
            other => Err(wrong_value_type(parameter, ParameterKind::Enum, other)),
        }
    }

    fn lookup(&self, id: &str) -> Result<(&ParameterId, &ParameterValue), ParameterError> {
        let parameter = ParameterId::try_new(id)
            .map_err(|_| ParameterError::InvalidParameterIdentifier { id: id.to_owned() })?;
        let value = self.values.get_key_value(&parameter).ok_or_else(|| {
            ParameterError::UnknownParameter {
                parameter: parameter.clone(),
            }
        })?;

        Ok(value)
    }
}

fn validate_parameter_type(
    parameter: &ParameterId,
    value_type: &ParameterType,
) -> Result<(), DefinitionError> {
    match value_type {
        ParameterType::I64 { minimum, maximum } => {
            validate_ordered_range(parameter, minimum, maximum)
        }
        ParameterType::U64 { minimum, maximum } => {
            validate_ordered_range(parameter, minimum, maximum)
        }
        ParameterType::F32 { minimum, maximum } => {
            validate_float_bound(parameter, *minimum, "minimum")?;
            validate_float_bound(parameter, *maximum, "maximum")?;
            validate_ordered_range(parameter, minimum, maximum)
        }
        ParameterType::F64 { minimum, maximum } => {
            validate_float_bound(parameter, *minimum, "minimum")?;
            validate_float_bound(parameter, *maximum, "maximum")?;
            validate_ordered_range(parameter, minimum, maximum)
        }
        ParameterType::Enum { options } => validate_enum_options(parameter, options),
        ParameterType::Bool
        | ParameterType::Vector2F64
        | ParameterType::Vector3F64
        | ParameterType::String => Ok(()),
    }
}

fn validate_ordered_range<T>(
    parameter: &ParameterId,
    minimum: &Option<T>,
    maximum: &Option<T>,
) -> Result<(), DefinitionError>
where
    T: PartialOrd,
{
    if let (Some(minimum), Some(maximum)) = (minimum, maximum) {
        if minimum > maximum {
            return Err(DefinitionError::InvalidNumericRange {
                parameter: parameter.clone(),
                message: "minimum is greater than maximum".to_owned(),
            });
        }
    }

    Ok(())
}

fn validate_float_bound<T>(
    parameter: &ParameterId,
    value: Option<T>,
    name: &str,
) -> Result<(), DefinitionError>
where
    T: FloatFinite,
{
    if value.is_some_and(|value| !value.is_finite()) {
        return Err(DefinitionError::InvalidNumericRange {
            parameter: parameter.clone(),
            message: format!("{name} must be finite"),
        });
    }

    Ok(())
}

fn validate_enum_options(
    parameter: &ParameterId,
    options: &[EnumOption],
) -> Result<(), DefinitionError> {
    if options.is_empty() {
        return Err(DefinitionError::InvalidEnumOptions {
            parameter: parameter.clone(),
            message: "enum options must not be empty".to_owned(),
        });
    }

    let mut ids = HashSet::new();
    for option in options {
        if !ids.insert(option.id()) {
            return Err(DefinitionError::InvalidEnumOptions {
                parameter: parameter.clone(),
                message: format!("duplicate option '{}'", option.id()),
            });
        }
    }

    Ok(())
}

fn validate_default(
    parameter: &ParameterId,
    value_type: &ParameterType,
    value: &ParameterValue,
) -> Result<(), DefinitionError> {
    if value.kind() != value_type.kind() {
        return Err(DefinitionError::InvalidDefault {
            parameter: parameter.clone(),
            message: format!("expected {:?}, found {:?}", value_type.kind(), value.kind()),
        });
    }

    let definition = ParameterDefinition {
        id: parameter.clone(),
        display_name: "temporary".into(),
        description: "".into(),
        value_type: value_type.clone(),
        default: None,
    };

    validate_parameter_value(&definition, value).map_err(|error| DefinitionError::InvalidDefault {
        parameter: parameter.clone(),
        message: error.to_string(),
    })
}

fn validate_parameter_value(
    definition: &ParameterDefinition,
    value: &ParameterValue,
) -> Result<(), ParameterError> {
    let parameter = definition.id().clone();
    if value.kind() != definition.value_type().kind() {
        return Err(ParameterError::WrongValueType {
            parameter,
            expected: definition.value_type().kind(),
            actual: value.kind(),
        });
    }

    match (definition.value_type(), value) {
        (ParameterType::Bool, ParameterValue::Bool(_)) => Ok(()),
        (ParameterType::I64 { minimum, maximum }, ParameterValue::I64(value)) => {
            validate_i64_value(&parameter, *value, *minimum, *maximum)
        }
        (ParameterType::U64 { minimum, maximum }, ParameterValue::U64(value)) => {
            validate_u64_value(&parameter, *value, *minimum, *maximum)
        }
        (ParameterType::F32 { minimum, maximum }, ParameterValue::F32(value)) => {
            validate_f32_value(&parameter, *value, *minimum, *maximum)
        }
        (ParameterType::F64 { minimum, maximum }, ParameterValue::F64(value)) => {
            validate_f64_value(&parameter, *value, *minimum, *maximum)
        }
        (ParameterType::Vector2F64, ParameterValue::Vector2F64(value)) => {
            if !value.is_finite() {
                return Err(ParameterError::NonFiniteNumericValue { parameter });
            }

            Ok(())
        }
        (ParameterType::Vector3F64, ParameterValue::Vector3F64(value)) => {
            if !value.is_finite() {
                return Err(ParameterError::NonFiniteNumericValue { parameter });
            }

            Ok(())
        }
        (ParameterType::String, ParameterValue::String(_)) => Ok(()),
        (ParameterType::Enum { options }, ParameterValue::Enum(value)) => {
            if options.iter().any(|option| option.id() == value) {
                return Ok(());
            }

            Err(ParameterError::UnknownEnumValue {
                parameter,
                value: value.clone(),
            })
        }
        _ => Err(ParameterError::WrongValueType {
            parameter,
            expected: definition.value_type().kind(),
            actual: value.kind(),
        }),
    }
}

fn validate_i64_value(
    parameter: &ParameterId,
    value: i64,
    minimum: Option<i64>,
    maximum: Option<i64>,
) -> Result<(), ParameterError> {
    if let Some(minimum) = minimum {
        if value < minimum {
            return Err(ParameterError::BelowMinimum {
                parameter: parameter.clone(),
                minimum: NumericValue::I64(minimum),
                actual: NumericValue::I64(value),
            });
        }
    }

    if let Some(maximum) = maximum {
        if value > maximum {
            return Err(ParameterError::AboveMaximum {
                parameter: parameter.clone(),
                maximum: NumericValue::I64(maximum),
                actual: NumericValue::I64(value),
            });
        }
    }

    Ok(())
}

fn validate_u64_value(
    parameter: &ParameterId,
    value: u64,
    minimum: Option<u64>,
    maximum: Option<u64>,
) -> Result<(), ParameterError> {
    if let Some(minimum) = minimum {
        if value < minimum {
            return Err(ParameterError::BelowMinimum {
                parameter: parameter.clone(),
                minimum: NumericValue::U64(minimum),
                actual: NumericValue::U64(value),
            });
        }
    }

    if let Some(maximum) = maximum {
        if value > maximum {
            return Err(ParameterError::AboveMaximum {
                parameter: parameter.clone(),
                maximum: NumericValue::U64(maximum),
                actual: NumericValue::U64(value),
            });
        }
    }

    Ok(())
}

fn validate_f32_value(
    parameter: &ParameterId,
    value: f32,
    minimum: Option<f32>,
    maximum: Option<f32>,
) -> Result<(), ParameterError> {
    if !value.is_finite() {
        return Err(ParameterError::NonFiniteNumericValue {
            parameter: parameter.clone(),
        });
    }

    if let Some(minimum) = minimum {
        if value < minimum {
            return Err(ParameterError::BelowMinimum {
                parameter: parameter.clone(),
                minimum: NumericValue::F32(minimum),
                actual: NumericValue::F32(value),
            });
        }
    }

    if let Some(maximum) = maximum {
        if value > maximum {
            return Err(ParameterError::AboveMaximum {
                parameter: parameter.clone(),
                maximum: NumericValue::F32(maximum),
                actual: NumericValue::F32(value),
            });
        }
    }

    Ok(())
}

fn validate_f64_value(
    parameter: &ParameterId,
    value: f64,
    minimum: Option<f64>,
    maximum: Option<f64>,
) -> Result<(), ParameterError> {
    if !value.is_finite() {
        return Err(ParameterError::NonFiniteNumericValue {
            parameter: parameter.clone(),
        });
    }

    if let Some(minimum) = minimum {
        if value < minimum {
            return Err(ParameterError::BelowMinimum {
                parameter: parameter.clone(),
                minimum: NumericValue::F64(minimum),
                actual: NumericValue::F64(value),
            });
        }
    }

    if let Some(maximum) = maximum {
        if value > maximum {
            return Err(ParameterError::AboveMaximum {
                parameter: parameter.clone(),
                maximum: NumericValue::F64(maximum),
                actual: NumericValue::F64(value),
            });
        }
    }

    Ok(())
}

fn wrong_value_type(
    parameter: &ParameterId,
    expected: ParameterKind,
    actual: &ParameterValue,
) -> ParameterError {
    ParameterError::WrongValueType {
        parameter: parameter.clone(),
        expected,
        actual: actual.kind(),
    }
}

trait FloatFinite: Copy {
    fn is_finite(self) -> bool;
}

impl FloatFinite for f32 {
    fn is_finite(self) -> bool {
        f32::is_finite(self)
    }
}

impl FloatFinite for f64 {
    fn is_finite(self) -> bool {
        f64::is_finite(self)
    }
}
