//! C ABI release validation and packaging for TerraKit.
//!
//! The tool does not build the library or publish a release. CI provides the
//! compiled native libraries and generated header, then invokes these commands.

mod archive;
mod digest;
mod native;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use digest::sha256;

type Result<T> = std::result::Result<T, String>;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Reads the workspace version; the manifest's package version is a simple string.
fn workspace_version() -> Result<String> {
    let manifest = read_text(&root().join("Cargo.toml"))?;
    let section = manifest
        .split("[workspace.package]")
        .nth(1)
        .ok_or("missing [workspace.package] section")?
        .split('\n')
        .take_while(|line| !line.trim().starts_with('['));
    for line in section {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "version" {
            let value = value.trim();
            return value
                .strip_prefix('"')
                .and_then(|v| v.split('"').next())
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| "invalid workspace version".to_owned());
        }
    }
    Err("missing workspace package version".to_owned())
}

/// Enumerates only functions with a `#[unsafe(no_mangle)]` C export attribute.
fn exported_functions() -> Result<Vec<String>> {
    let mut functions = BTreeSet::new();
    let source = root().join("engine/terrakit-c-api/src");
    for entry in fs::read_dir(&source).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let contents = read_text(&path)?;
        let mut no_mangle = false;
        for line in contents.lines() {
            let line = line.trim();
            if line == "#[unsafe(no_mangle)]" {
                no_mangle = true;
                continue;
            }
            if no_mangle {
                if let Some((_, suffix)) = line.split_once("extern \"C\" fn ") {
                    let name = suffix.split('(').next().unwrap_or("").trim();
                    if !name.starts_with("tk_") {
                        return Err(format!("unexpected C export in {}: {name}", path.display()));
                    }
                    functions.insert(name.to_owned());
                    no_mangle = false;
                } else if !line.is_empty() && !line.starts_with("///") && !line.starts_with("#[") {
                    return Err(format!(
                        "unrecognised C export in {}: {line}",
                        path.display()
                    ));
                }
            }
        }
    }
    if !functions.contains("tk_get_abi_version") || !functions.contains("tk_get_library_version") {
        return Err("missing native version exports".to_owned());
    }
    Ok(functions.into_iter().collect())
}

fn check_header(path: &Path) -> Result<Vec<String>> {
    let content = read_text(path)?;
    let exports = exported_functions()?;
    for name in &exports {
        if !content.contains(&format!("{name}(")) {
            return Err(format!("generated C header is missing export {name}"));
        }
    }
    for name in ["tk_scatter_point_t", "tk_scatter_points_view_t"] {
        if !content.contains(name) {
            return Err(format!("generated C header is missing {name}"));
        }
    }
    println!("Header includes all {} exported functions", exports.len());
    Ok(exports)
}

fn native_libraries(target: &str, artifacts: &Path) -> Result<(PathBuf, Vec<PathBuf>)> {
    let (shared, static_lib, optional) = match target {
        "x86_64-pc-windows-msvc" => ("terrakit.dll", "terrakit.lib", Some("terrakit.dll.lib")),
        "x86_64-apple-darwin" | "aarch64-apple-darwin" => {
            ("libterrakit.dylib", "libterrakit.a", None)
        }
        "x86_64-unknown-linux-gnu" => ("libterrakit.so", "libterrakit.a", None),
        _ => return Err(format!("unsupported release target: {target}")),
    };
    let shared = artifacts.join(shared);
    let mut libraries = vec![shared.clone(), artifacts.join(static_lib)];
    if let Some(optional) = optional {
        let optional = artifacts.join(optional);
        if optional.is_file() {
            libraries.push(optional);
        }
    }
    for library in &libraries {
        if !library.is_file() {
            return Err(format!("native library missing: {}", library.display()));
        }
    }
    Ok((shared, libraries))
}

/// Escape the fields that can appear in a release manifest JSON string.
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn manifest(version: &str, abi: &str, target: &str, files: &BTreeMap<String, Vec<u8>>) -> String {
    let checksums: Vec<String> = files
        .iter()
        .map(|(name, data)| format!("    {}: {}", json_string(name), json_string(&sha256(data))))
        .collect();
    format!(
        "{{\n  \"library_version\": {},\n  \"abi_version\": {},\n  \"target\": {},\n  \"files\": {{\n{}\n  }}\n}}\n",
        json_string(version),
        json_string(abi),
        json_string(target),
        checksums.join(",\n")
    )
}

fn package(target: &str, artifacts: &Path, header: &Path, output: &Path) -> Result<()> {
    let exports = check_header(header)?;
    let (shared, libraries) = native_libraries(target, artifacts)?;
    let version = workspace_version()?;
    let abi = native::verify(&shared, &exports, &version)?;
    let name = format!("terrakit-v{version}-{target}");
    let mut files = BTreeMap::new();
    files.insert("include/terrakit.h".to_owned(), read(header)?);
    files.insert("README.md".to_owned(), read(&root().join("docs/c-abi.md"))?);
    for library in libraries {
        let filename = library
            .file_name()
            .ok_or("invalid native library path")?
            .to_string_lossy();
        files.insert(format!("lib/{filename}"), read(&library)?);
    }
    let mut archive_files = BTreeMap::new();
    archive_files.insert(
        format!("{name}/manifest.json"),
        manifest(&version, &abi, target, &files).into_bytes(),
    );
    for (path, data) in files {
        archive_files.insert(format!("{name}/{path}"), data);
    }
    let zip = archive::write(&archive_files).map_err(|e| e.to_string())?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let archive_path = output.join(format!("{name}.zip"));
    fs::write(&archive_path, &zip).map_err(|e| e.to_string())?;
    let checksum_path = output.join(format!("{name}.zip.sha256"));
    fs::write(&checksum_path, format!("{}  {}.zip\n", sha256(&zip), name))
        .map_err(|e| e.to_string())?;
    println!(
        "Created {} and {}",
        archive_path.display(),
        checksum_path.display()
    );
    Ok(())
}

fn verify_archives(directory: &Path, expected: usize) -> Result<()> {
    let version = workspace_version()?;
    let mut archives = Vec::new();
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|ext| ext == "zip")
            && path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("terrakit-v"))
        {
            archives.push(path);
        }
    }
    if archives.len() != expected {
        return Err(format!(
            "expected {expected} release archives, found {}",
            archives.len()
        ));
    }
    let mut targets = BTreeSet::new();
    let mut release_abi: Option<String> = None;
    for path in archives {
        let filename = path
            .file_name()
            .ok_or("invalid archive name")?
            .to_string_lossy();
        let prefix = format!("terrakit-v{version}-");
        let target = filename
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(".zip"))
            .ok_or_else(|| format!("unexpected archive name: {filename}"))?;
        if !targets.insert(target.to_owned()) {
            return Err(format!("duplicate release target: {target}"));
        }
        let data = read(&path)?;
        let checksum = read_text(&path.with_extension("zip.sha256"))?;
        if checksum.trim() != format!("{}  {}", sha256(&data), filename) {
            return Err(format!("archive checksum mismatch: {filename}"));
        }
        let files = archive::read(&data).map_err(|e| format!("{filename}: {e}"))?;
        let root_name = format!("{prefix}{target}/");
        let manifest_path = format!("{root_name}manifest.json");
        let manifest_data = files
            .get(&manifest_path)
            .ok_or("archive is missing manifest.json")?;
        let manifest_text = std::str::from_utf8(manifest_data).map_err(|e| e.to_string())?;
        let abi = manifest_text
            .lines()
            .find_map(|line| {
                line.trim()
                    .strip_prefix("\"abi_version\": \"")?
                    .split('"')
                    .next()
            })
            .ok_or("release manifest is missing abi_version")?;
        if abi.is_empty() {
            return Err("release manifest has no ABI version".to_owned());
        }
        if let Some(previous) = &release_abi {
            if previous != abi {
                return Err(format!(
                    "release packages disagree on C ABI version: {previous} vs {abi}"
                ));
            }
        } else {
            release_abi = Some(abi.to_owned());
        }
        let mut contents = BTreeMap::new();
        for (path, content) in &files {
            let short = path
                .strip_prefix(&root_name)
                .ok_or("unexpected ZIP root directory")?;
            if short != "manifest.json" {
                contents.insert(short.to_owned(), content.clone());
            }
        }
        if !contents.contains_key("include/terrakit.h")
            || !contents.keys().any(|p| p.starts_with("lib/"))
        {
            return Err(format!("{filename}: missing C header or native libraries"));
        }
        let expected_manifest = manifest(&version, abi, target, &contents);
        if manifest_data != expected_manifest.as_bytes() {
            return Err(format!("{filename}: package manifest mismatch"));
        }
    }
    println!("Verified {expected} release archives, manifests, and checksums");
    Ok(())
}

fn option_path(args: &[String], key: &str) -> Result<PathBuf> {
    option(args, key).map(PathBuf::from)
}

fn option<'a>(args: &'a [String], key: &str) -> Result<&'a str> {
    let index = args
        .iter()
        .position(|arg| arg == key)
        .ok_or_else(|| format!("missing {key}"))?;
    args.get(index + 1)
        .map(String::as_str)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("missing value for {key}"))
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("version") => println!("{}", workspace_version()?),
        Some("check-header") => { check_header(&option_path(&args, "--header")?)?; }
        Some("package") => package(
            option(&args, "--target")?,
            &option_path(&args, "--artifact-dir")?,
            &option_path(&args, "--header")?,
            &option_path(&args, "--output-dir")?,
        )?,
        Some("verify-archives") => verify_archives(
            &option_path(&args, "--directory")?,
            option(&args, "--expected")?.parse().map_err(|_| "--expected must be an integer")?,
        )?,
        _ => return Err("usage: cargo run -p terrakit-xtask -- [version | check-header --header PATH | package --target TRIPLE --artifact-dir PATH --header PATH --output-dir PATH | verify-archives --directory PATH --expected N]".to_owned()),
    }
    Ok(())
}

fn main() {
    if let Err(message) = run() {
        eprintln!("error: {message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_package_manifest() {
        assert_eq!(workspace_version().unwrap(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn discovers_scatter_export() {
        assert!(
            exported_functions()
                .unwrap()
                .contains(&"tk_generation_result_get_scatter_points".to_owned())
        );
    }

    #[test]
    fn package_manifest_hashes_file_bytes() {
        let files = BTreeMap::from([("include/terrakit.h".to_owned(), b"abc".to_vec())]);
        let body = manifest("0.0.3", "0.2.0", "test", &files);
        assert!(body.contains(&sha256(b"abc")));
    }
}
