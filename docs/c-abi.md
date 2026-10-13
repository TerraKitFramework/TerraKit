# TerraKit C ABI

TerraKit provides a C-compatible interface from the `terrakit-c-api` Rust crate. It exposes built-in stage discovery, stage construction, ordered pipelines, synchronous region generation, and immutable resource views. It does not allow C callers to implement Rust stages or mutate resources through the ABI.

## Release package

A release for each supported target contains:

```text
terrakit-v<library-version>-<target>/
├── include/terrakit.h
├── lib/<shared-library>
├── lib/<static-library>
├── README.md
└── manifest.json
```

On Windows, the package also includes `terrakit.dll.lib` when Cargo produces it. The manifest lists the library version, the **independent** C ABI version, the target triple, and SHA-256 hashes for its files. The matching `.zip.sha256` file accompanies each ZIP on the GitHub Release page.

Targets released by CI:

| Target | Shared library | Static library |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `libterrakit.so` | `libterrakit.a` |
| `x86_64-pc-windows-msvc` | `terrakit.dll` | `terrakit.lib` |
| `x86_64-apple-darwin` | `libterrakit.dylib` | `libterrakit.a` |
| `aarch64-apple-darwin` | `libterrakit.dylib` | `libterrakit.a` |

Static linking may require additional system libraries supplied by the platform toolchain. The Windows `.lib` named `terrakit.dll.lib`, when present, is the DLL import library; `terrakit.lib` is the Rust static library.

## Linking and version checks

Add `include` to your compiler's include path and link against the appropriate native binary. On Windows, define `TK_USE_SHARED` when compiling callers that import functions from the DLL. `TK_BUILD_SHARED` is intended for building the DLL itself; callers should not define it.

Use `tk_get_abi_version` to determine the ABI contract and `tk_get_library_version` to read the TerraKit library version. They are different versions: a library patch release does not imply an ABI version change, and an ABI change must be reviewed separately.

## Ownership and data views

Owned objects are opaque handles. Destroy a handle with its matching `tk_*_destroy` function; successful destruction clears the caller's handle pointer. Pipelines and runtimes have the ownership transfer rules documented on their functions in the generated header.

Generated data is owned by a generation-result handle. Returned resource views borrow memory from that result, and must not be accessed after it is destroyed. No resource data is allocated for the caller to `free` directly. Schema strings are borrowed from the registry handle that provides them.

`tk_string_view_t` carries a UTF-8 byte pointer and explicit byte length; it is not guaranteed to end in a NUL byte. Functions report errors through `tk_status_t`. `tk_last_error_message_copy` copies the thread-local error message into caller-owned memory.

For exact argument types and error codes, use the generated `terrakit.h` corresponding to the library release. Do not pair a header from one ABI version with binaries from another without checking compatibility.

## Build from source

```sh
cargo build --locked --release -p terrakit-c-api --all-features
cargo install cbindgen --version 0.29.0 --locked
# Create bindings/c/include first.
cbindgen --config engine/terrakit-c-api/cbindgen.toml --crate terrakit-c-api --output bindings/c/include/terrakit.h
cargo run --locked -p terrakit-xtask -- check-header --header bindings/c/include/terrakit.h
```

The committed sources contain the ABI declarations and `cbindgen.toml`; generated headers, binaries, release archives, and checksums are not version-controlled.

## Release tooling

`terrakit-xtask` is a private, dependency-free Rust workspace crate. GitHub Actions builds the native libraries and generates the C header. `xtask` validates header exports, verifies exported functions and the library version in the built shared library, and creates target-specific ZIP archives with a manifest and SHA-256 checksum files. Archive entries are stored without compression. The release workflow verifies all four packages before publishing.
