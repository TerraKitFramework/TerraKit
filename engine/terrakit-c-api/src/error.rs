//! Thread-local diagnostics and shared panic containment for exported calls.

use std::{
    cell::RefCell,
    ffi::c_char,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
};

use terrakit_core::CoreError;
use terrakit_pipeline::{BindingError, DefinitionError, ParameterError, PipelineAssemblyError};
use terrakit_runtime::RuntimeError;

use crate::{
    TK_STATUS_BUFFER_TOO_SMALL, TK_STATUS_CONFIGURATION_ERROR, TK_STATUS_GENERATION_ERROR,
    TK_STATUS_INTERNAL_ERROR, TK_STATUS_INVALID_ARGUMENT, TK_STATUS_INVALID_UTF8,
    TK_STATUS_NOT_FOUND, TK_STATUS_NULL_ARGUMENT, TK_STATUS_OK, TK_STATUS_OUT_OF_RANGE,
    TK_STATUS_SCHEMA_VERSION_MISMATCH, TK_STATUS_TYPE_MISMATCH, TkStatus,
};

thread_local! {
    static LAST_ERROR_MESSAGE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Internal ABI error with a stable public status and detailed diagnostic text.
#[derive(Debug, Clone)]
pub struct AbiError {
    status: TkStatus,
    message: String,
}

impl AbiError {
    pub(crate) fn new(status: TkStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub(crate) fn null_argument(name: &'static str) -> Self {
        Self::new(
            TK_STATUS_NULL_ARGUMENT,
            format!("required argument '{name}' was null"),
        )
    }

    pub(crate) fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(TK_STATUS_INVALID_ARGUMENT, message)
    }

    pub(crate) fn invalid_utf8(name: &'static str) -> Self {
        Self::new(
            TK_STATUS_INVALID_UTF8,
            format!("string argument '{name}' is not valid UTF-8"),
        )
    }

    pub(crate) fn out_of_range(message: impl Into<String>) -> Self {
        Self::new(TK_STATUS_OUT_OF_RANGE, message)
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::new(TK_STATUS_NOT_FOUND, message)
    }

    pub(crate) fn type_mismatch(message: impl Into<String>) -> Self {
        Self::new(TK_STATUS_TYPE_MISMATCH, message)
    }

    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::new(TK_STATUS_CONFIGURATION_ERROR, message)
    }

    pub(crate) fn generation(message: impl Into<String>) -> Self {
        Self::new(TK_STATUS_GENERATION_ERROR, message)
    }

    fn status(&self) -> TkStatus {
        self.status
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Executes one exported operation behind last-error clearing and panic containment.
pub(crate) fn ffi_guard(operation: impl FnOnce() -> Result<(), AbiError>) -> TkStatus {
    clear_last_error_storage();

    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => TK_STATUS_OK,
        Ok(Err(error)) => {
            set_last_error(error.message());
            error.status()
        }
        Err(payload) => {
            let detail = if let Some(message) = payload.downcast_ref::<&'static str>() {
                *message
            } else if let Some(message) = payload.downcast_ref::<String>() {
                message.as_str()
            } else {
                "non-string panic payload"
            };
            set_last_error(format!("panic caught at TerraKit C ABI boundary: {detail}"));
            TK_STATUS_INTERNAL_ERROR
        }
    }
}

/// Clears the thread-local ABI diagnostic message.
#[unsafe(no_mangle)]
pub extern "C" fn tk_clear_last_error() {
    let _ = catch_unwind(AssertUnwindSafe(clear_last_error_storage));
}

/// Copies the thread-local ABI diagnostic message into a caller buffer.
///
/// # Safety
///
/// Any non-null `buffer` must point to caller-owned writable storage of at
/// least `capacity` bytes, and any non-null `out_required` must point to
/// caller-owned writable storage for one `usize`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tk_last_error_message_copy(
    buffer: *mut c_char,
    capacity: usize,
    out_required: *mut usize,
) -> TkStatus {
    match catch_unwind(AssertUnwindSafe(|| {
        last_error_message_copy_impl(buffer, capacity, out_required)
    })) {
        Ok(status) => status,
        Err(_) => TK_STATUS_INTERNAL_ERROR,
    }
}

fn last_error_message_copy_impl(
    buffer: *mut c_char,
    capacity: usize,
    out_required: *mut usize,
) -> TkStatus {
    if out_required.is_null() {
        return TK_STATUS_NULL_ARGUMENT;
    }

    let message = LAST_ERROR_MESSAGE.with(|stored| stored.borrow().clone().unwrap_or_default());
    let required = message.len().saturating_add(1);

    // SAFETY: out_required was checked for null and points to caller-owned writable storage.
    unsafe {
        *out_required = required;
    }

    if buffer.is_null() {
        return if capacity == 0 {
            TK_STATUS_OK
        } else {
            TK_STATUS_NULL_ARGUMENT
        };
    }

    if capacity < required {
        return TK_STATUS_BUFFER_TOO_SMALL;
    }

    // SAFETY: buffer is non-null, capacity is at least message length plus one
    // trailing NUL byte, and source/destination do not overlap.
    unsafe {
        ptr::copy_nonoverlapping(message.as_ptr().cast::<c_char>(), buffer, message.len());
        *buffer.add(message.len()) = 0;
    }

    TK_STATUS_OK
}

pub(crate) fn definition_error(context: impl AsRef<str>, error: DefinitionError) -> AbiError {
    AbiError::invalid_argument(format!("{}: {error}", context.as_ref()))
}

pub(crate) fn binding_error(context: impl AsRef<str>, error: BindingError) -> AbiError {
    let status = match error {
        BindingError::UnknownInputPort { .. } | BindingError::UnknownOutputPort { .. } => {
            TK_STATUS_NOT_FOUND
        }
        BindingError::InvalidPortIdentifier { .. } => TK_STATUS_INVALID_ARGUMENT,
        BindingError::DuplicateInputBinding { .. }
        | BindingError::DuplicateOutputBinding { .. }
        | BindingError::DuplicateOutputResource { .. }
        | BindingError::InputOutputAlias { .. }
        | BindingError::MissingRequiredInput { .. }
        | BindingError::MissingOutput { .. } => TK_STATUS_CONFIGURATION_ERROR,
    };
    AbiError::new(status, format!("{}: {error}", context.as_ref()))
}

pub(crate) fn parameter_error(context: impl AsRef<str>, error: ParameterError) -> AbiError {
    let status = match error {
        ParameterError::UnknownParameter { .. } | ParameterError::UnknownEnumValue { .. } => {
            TK_STATUS_NOT_FOUND
        }
        ParameterError::WrongValueType { .. } => TK_STATUS_TYPE_MISMATCH,
        ParameterError::InvalidParameterIdentifier { .. }
        | ParameterError::NonFiniteNumericValue { .. } => TK_STATUS_INVALID_ARGUMENT,
        ParameterError::MissingRequiredParameter { .. }
        | ParameterError::BelowMinimum { .. }
        | ParameterError::AboveMaximum { .. } => TK_STATUS_CONFIGURATION_ERROR,
    };
    AbiError::new(status, format!("{}: {error}", context.as_ref()))
}

pub(crate) fn assembly_error(error: PipelineAssemblyError) -> AbiError {
    let status = match &error {
        PipelineAssemblyError::UnknownStageType { .. } => TK_STATUS_NOT_FOUND,
        PipelineAssemblyError::SchemaVersionMismatch { .. } => TK_STATUS_SCHEMA_VERSION_MISMATCH,
        PipelineAssemblyError::WrongInputResourceKind { .. } => TK_STATUS_TYPE_MISMATCH,
        PipelineAssemblyError::Parameter { source, .. } => {
            return parameter_error("pipeline assembly failed", source.clone());
        }
        PipelineAssemblyError::Binding { source, .. } => {
            return binding_error("pipeline assembly failed", source.clone());
        }
        PipelineAssemblyError::DuplicateStageId { .. }
        | PipelineAssemblyError::MissingInputResource { .. }
        | PipelineAssemblyError::OutputResourceAlreadyAssigned { .. }
        | PipelineAssemblyError::InputOutputAlias { .. }
        | PipelineAssemblyError::StageBuild { .. }
        | PipelineAssemblyError::StageContractMismatch { .. } => TK_STATUS_CONFIGURATION_ERROR,
    };
    AbiError::new(status, format!("pipeline assembly failed: {error}"))
}

pub(crate) fn runtime_error(error: RuntimeError) -> AbiError {
    match &error {
        RuntimeError::LayoutMismatch { .. } => {
            AbiError::type_mismatch(format!("runtime generation failed: {error}"))
        }
        RuntimeError::RegionResolution { source } => {
            let status = match source {
                CoreError::DimensionOverflow
                | CoreError::UnsupportedLod { .. }
                | CoreError::LodScaleOverflow { .. } => TK_STATUS_OUT_OF_RANGE,
                CoreError::ZeroDimension
                | CoreError::InvalidSpacing { .. }
                | CoreError::LodNotDivisible { .. }
                | CoreError::NonFiniteRegionOrigin
                | CoreError::InvalidTransform
                | CoreError::InvalidBufferLength { .. }
                | CoreError::CoordinateOutOfBounds2 { .. }
                | CoreError::CoordinateOutOfBounds3 { .. }
                | CoreError::NonFiniteHeightSample
                | CoreError::NonFiniteDensitySample
                | CoreError::InvalidHeightAxis
                | CoreError::NonFiniteMeshOrigin
                | CoreError::MeshVertexCountOverflow { .. }
                | CoreError::NonFiniteMeshAttribute { .. }
                | CoreError::InvalidMeshIndex { .. }
                | CoreError::InvalidTriangleIndexCount { .. }
                | CoreError::InvalidAttributeLength { .. } => TK_STATUS_INVALID_ARGUMENT,
            };
            AbiError::new(status, format!("runtime generation failed: {error}"))
        }
        RuntimeError::PipelineExecution { .. } => {
            AbiError::generation(format!("runtime generation failed: {error}"))
        }
    }
}

fn set_last_error(message: impl AsRef<str>) {
    LAST_ERROR_MESSAGE.with(|stored| {
        *stored.borrow_mut() = Some(message.as_ref().to_owned());
    });
}

fn clear_last_error_storage() {
    LAST_ERROR_MESSAGE.with(|stored| {
        *stored.borrow_mut() = None;
    });
}
