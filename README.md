# TerraKit

[![CI](https://github.com/Crinja/TerraKit/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/Crinja/TerraKit/actions/workflows/ci.yml)
[![Releases](https://img.shields.io/github/v/release/Crinja/TerraKit?filter=terrakit-v*&label=TerraKit)](https://github.com/Crinja/TerraKit/releases)

TerraKit is a Rust library for deterministic, engine-independent terrain generation. It provides region and grid data types, generation algorithms, composable pipeline stages, synchronous execution, and a C ABI for applications written outside Rust.

TerraKit does not ship editor, game-engine, or command-line consumer integrations in this repository. The C ABI is a library interface, not a separate consumer application.

## Crates

| Crate | Responsibility |
| --- | --- |
| `terrakit-core` | Regions, grids, transforms, terrain resources, and seeds |
| `terrakit-algorithms` | Noise, interpolation, mesh helpers, and scattering algorithms |
| `terrakit-pipeline` | Stage definitions, resource bindings, and ordered pipeline execution |
| `terrakit-builtins` | Built-in generation stages and their parameter schemas |
| `terrakit-runtime` | Region generation requests and synchronous runtime execution |
| `terrakit-c-api` | Native C-compatible functions, opaque handles, and borrowed result views |

Workspace Rust packages are versioned together in `[workspace.package]` in `Cargo.toml`. The exported C ABI has a **separate** version returned by `tk_get_abi_version` in `engine/terrakit-c-api/src/version.rs`.

## Build and test

Install the Rust toolchain specified by `rust-toolchain.toml` (Rust 1.87) and run these commands from the repository root. **Make is not required.** The `terrakit-xtask` workspace crate contains development-only C-ABI release tooling.

```sh
cargo build --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

To build both the shared and static native libraries:

```sh
cargo build --locked --release -p terrakit-c-api --all-features
```

The build creates `terrakit.dll` and `terrakit.lib` on Windows, `libterrakit.so` and `libterrakit.a` on Linux, or `libterrakit.dylib` and `libterrakit.a` on macOS. The path is `target/release` for a native build; when passing `--target`, it is `target/<target-triple>/release`.

### Generate the C header locally

`terrakit.h` is **generated**, not committed. Install the pinned generator and create it from the Rust ABI declarations:

```sh
cargo install cbindgen --version 0.29.0 --locked
cbindgen --config engine/terrakit-c-api/cbindgen.toml --crate terrakit-c-api --output bindings/c/include/terrakit.h
cargo run --locked -p terrakit-xtask -- check-header --header bindings/c/include/terrakit.h
```

Create `bindings/c/include` first if it does not exist (`mkdir -p bindings/c/include` on Linux/macOS, or `New-Item -ItemType Directory -Force bindings/c/include` in PowerShell). The command requires no Python installation. These commands work from the repository root and do not require Make. For the exported functions, lifetime rules, Windows DLL import macro, and packaged assets, see [C ABI documentation](docs/c-abi.md).

## Branches and releases

Only two **long-lived** branches are used:

- **`dev`** collects reviewed feature branches. Open feature pull requests against `dev` and keep `dev` passing CI.
- **`main`** is the release branch. Promote `dev` to `main` with a pull request after deciding on the next workspace package version. A new package version on `main` triggers a C-ABI release.

Short-lived feature branches (for example, `feature/object-scattering`) are allowed; the two-branch rule applies to persistent branches. Avoid committing generated headers and compiled artifacts. Configure GitHub branch protection to require the **CI passed** check for pull requests into both `dev` and `main` and disallow direct pushes to `main`.

The release workflow on `main`:

1. Reads the shared library version from `Cargo.toml` and checks for an existing `terrakit-v<version>` GitHub Release. Previously released versions are skipped; a tag without a release can only be retried at its original commit.
2. Runs Rust format/lint/tests, regenerates the C header, and compile-checks that header as both C and C++.
3. Builds and verifies the exported C ABI on Linux x86_64, Windows x86_64, macOS x86_64, and macOS ARM64.
4. Publishes a GitHub Release containing a platform ZIP and a SHA-256 checksum per target. Each ZIP has `include/terrakit.h`, the shared and static native libraries, a package manifest, and C ABI usage notes.

**Bump `workspace.package.version` and update `Cargo.lock` for a new release.** If you change the C ABI contract, review and intentionally update its independent version in `version.rs` and relevant ABI tests. Do not move an existing release tag to a newer commit to reuse a version.

The `terrakit-xtask` commands are `version`, `check-header`, `package`, and `verify-archives`. The packaging command loads the newly built shared library to validate its exported functions and version information. Packages use portable, uncompressed ZIP entries to avoid external archiving tools.

See [the release workflow](.github/workflows/release.yml) and [C ABI distribution notes](docs/c-abi.md) for the precise checks and package contents.
