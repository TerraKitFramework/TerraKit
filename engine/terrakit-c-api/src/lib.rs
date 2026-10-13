//! TerraKit embedding C ABI.
//!
//! This crate exposes TerraKit to C-compatible embedding hosts through a stable
//! handle-and-view ABI. It is the embedding boundary for discovering built-in
//! stages, configuring ordered stage constructions, creating synchronous
//! runtimes, generating regions, and reading immutable generated resources.
//!
//! This is not the external plugin ABI. Foreign code cannot implement
//! `TerrainStage`, register callback stages, provide host allocators, or mutate
//! TerraKit resource sets through this boundary.
//!
//! # Ownership
//!
//! All owned objects are returned as opaque handles and must be destroyed with
//! their matching `tk_*_destroy` function. Destroy functions accept
//! pointer-to-pointer handles, set successful destructions to null, and treat an
//! already-null inner handle as success.
//!
//! # Results and Views
//!
//! A generation result owns its resources independently from the runtime that
//! produced it. Resource views contain immutable zero-copy pointers borrowed
//! from the result and remain valid only until that result is destroyed.
//!
//! # Threading
//!
//! Registry and result queries may run concurrently when the caller keeps the
//! handle alive and prevents concurrent destruction. Construction, assembler,
//! pipeline, and runtime handles are single-operation-at-a-time values.
//!
//! # Errors and Panics
//!
//! Every fallible exported function returns a stable `tk_status_t`. Detailed
//! diagnostics are stored in thread-local error storage and may be copied with
//! `tk_last_error_message_copy`. All exported calls run behind `catch_unwind`,
//! so Rust panics are converted to `TK_STATUS_INTERNAL_ERROR` instead of
//! crossing the C boundary.
//!
//! # Pipeline Construction
//!
//! Callers supply ordered stage constructions, parameters, and resource-key
//! bindings. The ABI does not model visual graphs or persist graph state.

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

mod construction;
mod error;
mod ffi;
mod pipeline;
mod registry;
mod result;
mod runtime;
mod status;
mod types;
mod version;

pub use construction::*;
pub use error::{tk_clear_last_error, tk_last_error_message_copy};
pub use pipeline::*;
pub use registry::*;
pub use result::*;
pub use runtime::*;
pub use status::*;
pub use types::*;
pub use version::*;

/// Invokes the shared panic guard for ABI boundary tests without exporting a C symbol.
#[doc(hidden)]
pub fn tk_test_invoke_panic_guard() -> TkStatus {
    error::ffi_guard(|| -> Result<(), error::AbiError> {
        panic!("intentional ABI panic test");
    })
}
