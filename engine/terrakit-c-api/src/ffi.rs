//! Focused raw-pointer, string-view, and handle helpers for the ABI boundary.

use std::{slice, str};

use crate::{TK_FALSE, TK_TRUE, TkBool, error::AbiError, types::TkStringView};

pub(crate) fn out_ref<'a, T>(pointer: *mut T, name: &'static str) -> Result<&'a mut T, AbiError> {
    if pointer.is_null() {
        return Err(AbiError::null_argument(name));
    }

    // SAFETY: pointer was checked for null. The caller contract requires a live,
    // writable object of the correct type for output arguments.
    Ok(unsafe { &mut *pointer })
}

pub(crate) fn ref_from_ptr<'a, T, Opaque>(
    pointer: *const Opaque,
    name: &'static str,
) -> Result<&'a T, AbiError> {
    if pointer.is_null() {
        return Err(AbiError::null_argument(name));
    }

    // SAFETY: pointer was checked for null. The caller contract requires a live
    // TerraKit handle of the correct concrete type.
    Ok(unsafe { &*(pointer.cast::<T>()) })
}

pub(crate) fn mut_from_ptr<'a, T, Opaque>(
    pointer: *mut Opaque,
    name: &'static str,
) -> Result<&'a mut T, AbiError> {
    if pointer.is_null() {
        return Err(AbiError::null_argument(name));
    }

    // SAFETY: pointer was checked for null. The caller contract requires a live,
    // uniquely borrowed TerraKit handle of the correct concrete type.
    Ok(unsafe { &mut *(pointer.cast::<T>()) })
}

pub(crate) fn handle_slot<'a, Opaque>(
    pointer: *mut *mut Opaque,
    name: &'static str,
) -> Result<&'a mut *mut Opaque, AbiError> {
    if pointer.is_null() {
        return Err(AbiError::null_argument(name));
    }

    // SAFETY: pointer was checked for null. The caller contract requires a live,
    // writable handle slot.
    Ok(unsafe { &mut *pointer })
}

pub(crate) fn destroy_handle<Handle, Opaque>(
    pointer: *mut *mut Opaque,
    name: &'static str,
) -> Result<(), AbiError> {
    let slot = handle_slot(pointer, name)?;
    if slot.is_null() {
        return Ok(());
    }

    // SAFETY: slot contains a live handle allocation of the matching concrete
    // type by the caller contract. This function takes ownership exactly once.
    unsafe {
        drop(Box::from_raw((*slot).cast::<Handle>()));
    }
    *slot = std::ptr::null_mut();
    Ok(())
}

pub(crate) fn take_handle<Handle, Opaque>(
    slot: &mut *mut Opaque,
    name: &'static str,
) -> Result<Box<Handle>, AbiError> {
    if slot.is_null() {
        return Err(AbiError::null_argument(name));
    }

    // SAFETY: slot contains a live handle allocation of the matching concrete
    // type by the caller contract. The slot is immediately nulled so the
    // allocation cannot be taken twice through this variable.
    let handle = unsafe { Box::from_raw((*slot).cast::<Handle>()) };
    *slot = std::ptr::null_mut();
    Ok(handle)
}

pub(crate) fn string_view_to_str<'a>(
    view: TkStringView,
    name: &'static str,
) -> Result<&'a str, AbiError> {
    if view.length == 0 {
        if view.data.is_null() {
            return Ok("");
        }

        // SAFETY: creating a zero-length slice from a non-null pointer is valid;
        // the branch above avoids constructing a slice from a null pointer.
        let bytes = unsafe { slice::from_raw_parts(view.data.cast::<u8>(), 0) };
        return str::from_utf8(bytes).map_err(|_| AbiError::invalid_utf8(name));
    }

    if view.data.is_null() {
        return Err(AbiError::null_argument(name));
    }

    // SAFETY: data is non-null and the caller contract requires `length` bytes
    // to be readable for the duration of the call.
    let bytes = unsafe { slice::from_raw_parts(view.data.cast::<u8>(), view.length) };
    str::from_utf8(bytes).map_err(|_| AbiError::invalid_utf8(name))
}

pub(crate) fn require_bool(value: TkBool, name: &'static str) -> Result<bool, AbiError> {
    match value {
        TK_FALSE => Ok(false),
        TK_TRUE => Ok(true),
        other => Err(AbiError::invalid_argument(format!(
            "boolean argument '{name}' had invalid value {other}"
        ))),
    }
}

pub(crate) fn require_zero_reserved(bytes: &[u8], name: &'static str) -> Result<(), AbiError> {
    if bytes.iter().any(|byte| *byte != 0) {
        return Err(AbiError::invalid_argument(format!(
            "reserved field '{name}' must be zero"
        )));
    }

    Ok(())
}

pub(crate) fn slice_pointer<T>(slice: &[T]) -> *const T {
    if slice.is_empty() {
        std::ptr::null()
    } else {
        slice.as_ptr()
    }
}
