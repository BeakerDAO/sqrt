use radix_engine::utils::extract_schema;
use radix_engine_interface::schema::PackageSchema;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageSchema) {
    // Build
    let status = Command::new("cargo")
        .current_dir(package_dir.as_ref())
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();
    if !status.success() {
        panic!("Failed to compile package: {:?}", package_dir.as_ref());
    }

    // Find wasm path
    let mut cargo = package_dir.as_ref().to_owned();
    cargo.push("Cargo.toml");
    let wasm_name = if cargo.exists() {
        let content = fs::read_to_string(&cargo).expect("Failed to read the Cargo.toml file");
        extract_crate_name(&content)
            .expect("Failed to extract crate name from the Cargo.toml file")
            .replace("-", "_")
    } else {
        // file name
        package_dir
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
            .replace("-", "_")
    };
    let mut path = PathBuf::from_str(&get_cargo_target_directory(&cargo)).unwrap(); // Infallible;
    path.push("wasm32-unknown-unknown");
    path.push("release");
    path.push(wasm_name);
    path.set_extension("wasm");

    // Extract schema
    let code = fs::read(&path).unwrap_or_else(|err| {
        panic!(
            "Failed to read built WASM from path {:?} - {:?}",
            &path, err
        )
    });
    let schema = extract_schema(&code).unwrap();

    (code, schema)
}

// A naive pattern matching to find the crate name.
fn extract_crate_name(mut content: &str) -> Result<String, ()> {
    let idx = content.find("name").ok_or(())?;
    content = &content[idx + 4..];

    let idx = content.find('"').ok_or(())?;
    content = &content[idx + 1..];

    let end = content.find('"').ok_or(())?;
    Ok(content[..end].to_string())
}

fn get_cargo_target_directory(manifest_path: impl AsRef<OsStr>) -> String {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()
        .expect("Failed to call cargo metadata");
    if output.status.success() {
        let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .expect("Failed to parse cargo metadata");
        let target_directory = parsed
            .as_object()
            .and_then(|o| o.get("target_directory"))
            .and_then(|o| o.as_str())
            .expect("Failed to parse target_directory from cargo metadata");
        target_directory.to_owned()
    } else {
        panic!("Cargo metadata call was not successful");
    }
}
