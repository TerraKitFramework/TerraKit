//! ABI and library version exports.

use std::ffi::c_char;

use crate::{TkStatus, error::ffi_guard, ffi::out_ref, types::TkStringView, types::TkVersion};

const ABI_VERSION: TkVersion = TkVersion {
    major: 0,
    minor: 2,
    patch: 0,
};

const LIBRARY_VERSION_BYTES: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();

/// Writes the embedding ABI version, currently `0.2.0`.
#[unsafe(no_mangle)]
pub extern "C" fn tk_get_abi_version(out_version: *mut TkVersion) -> TkStatus {
    ffi_guard(|| {
        let out = out_ref(out_version, "out_version")?;
        *out = ABI_VERSION;
        Ok(())
    })
}

/// Writes the TerraKit Rust package version string.
#[unsafe(no_mangle)]
pub extern "C" fn tk_get_library_version(out_version: *mut TkStringView) -> TkStatus {
    ffi_guard(|| {
        let out = out_ref(out_version, "out_version")?;
        *out = TkStringView {
            data: LIBRARY_VERSION_BYTES.as_ptr().cast::<c_char>(),
            length: LIBRARY_VERSION_BYTES.len() - 1,
        };
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TK_STATUS_OK;

    #[test]
    fn abi_version_is_current_development_version() {
        assert_eq!(
            (ABI_VERSION.major, ABI_VERSION.minor, ABI_VERSION.patch),
            (0, 2, 0)
        );
    }

    #[test]
    fn success_constant_matches_public_contract() {
        assert_eq!(TK_STATUS_OK, 0);
    }
}
