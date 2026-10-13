//! Load the built shared library and validate its public symbols and versions.

use std::ffi::{CString, c_char, c_void};
use std::path::Path;

#[repr(C)]
#[derive(Default)]
struct TkVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

#[repr(C)]
struct TkStringView {
    data: *const c_char,
    length: usize,
}

impl Default for TkStringView {
    fn default() -> Self {
        Self {
            data: std::ptr::null(),
            length: 0,
        }
    }
}

#[cfg(unix)]
mod os {
    use super::*;
    #[cfg_attr(target_os = "linux", link(name = "dl"))]
    unsafe extern "C" {
        fn dlopen(filename: *const c_char, flags: i32) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> i32;
    }
    pub fn open(path: &Path) -> Result<*mut c_void, String> {
        use std::os::unix::ffi::OsStrExt;
        let path = CString::new(path.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
        let handle = unsafe { dlopen(path.as_ptr(), 2) }; // RTLD_NOW
        if handle.is_null() {
            return Err(format!(
                "cannot load native library: {}",
                path.to_string_lossy()
            ));
        }
        Ok(handle)
    }
    pub fn symbol(handle: *mut c_void, name: &CString) -> *mut c_void {
        unsafe { dlsym(handle, name.as_ptr()) }
    }
    pub fn close(handle: *mut c_void) {
        unsafe { dlclose(handle) };
    }
}

#[cfg(windows)]
mod os {
    use super::*;
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryW(path: *const u16) -> *mut c_void;
        fn GetProcAddress(handle: *mut c_void, name: *const c_char) -> *mut c_void;
        fn FreeLibrary(handle: *mut c_void) -> i32;
    }
    pub fn open(path: &Path) -> Result<*mut c_void, String> {
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let handle = unsafe { LoadLibraryW(path.as_ptr()) };
        if handle.is_null() {
            return Err(format!(
                "cannot load native library: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(handle)
    }
    pub fn symbol(handle: *mut c_void, name: &CString) -> *mut c_void {
        unsafe { GetProcAddress(handle, name.as_ptr()) }
    }
    pub fn close(handle: *mut c_void) {
        unsafe { FreeLibrary(handle) };
    }
}

struct Library(*mut c_void);

impl Library {
    fn open(path: &Path) -> Result<Self, String> {
        let path = path
            .canonicalize()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        // SAFETY: The library path points at a freshly built TerraKit shared library.
        Ok(Self(os::open(&path)?))
    }

    fn symbol(&self, name: &str) -> Result<*mut c_void, String> {
        let name_c = CString::new(name).map_err(|e| e.to_string())?;
        // SAFETY: The handle stays loaded until Library::drop.
        let address = os::symbol(self.0, &name_c);
        if address.is_null() {
            return Err(format!("native library is missing export: {name}"));
        }
        Ok(address)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: The handle is owned and is dropped once.
        os::close(self.0);
    }
}

/// Checks all declared C header exports and returns the library's reported ABI version.
pub fn verify(path: &Path, exports: &[String], version: &str) -> Result<String, String> {
    let library = Library::open(path)?;
    for name in exports {
        library.symbol(name)?;
    }
    type AbiFn = unsafe extern "C" fn(*mut TkVersion) -> i32;
    type LibraryFn = unsafe extern "C" fn(*mut TkStringView) -> i32;

    // SAFETY: Both signatures match the two exports from version.rs; their symbols
    // were found in the loaded library and remain valid until it is unloaded.
    let abi_fn: AbiFn = unsafe { std::mem::transmute(library.symbol("tk_get_abi_version")?) };
    let library_fn: LibraryFn =
        unsafe { std::mem::transmute(library.symbol("tk_get_library_version")?) };
    let mut abi = TkVersion::default();
    let mut text = TkStringView::default();
    // SAFETY: The pointers refer to correctly laid out, writable C structures.
    let abi_status = unsafe { abi_fn(&mut abi) };
    let library_status = unsafe { library_fn(&mut text) };
    if abi_status != 0 || library_status != 0 || text.data.is_null() || text.length > 100 {
        return Err("native version exports returned invalid data".to_owned());
    }
    // SAFETY: The ABI guarantees a borrowed string view valid while the library is loaded.
    let bytes = unsafe { std::slice::from_raw_parts(text.data.cast::<u8>(), text.length) };
    let found = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    if found != version {
        return Err(format!(
            "library version mismatch: expected {version}, got {found}"
        ));
    }
    let abi_version = format!("{}.{}.{}", abi.major, abi.minor, abi.patch);
    println!(
        "Verified {} native exports; library {found}; ABI {abi_version}",
        exports.len()
    );
    Ok(abi_version)
}
