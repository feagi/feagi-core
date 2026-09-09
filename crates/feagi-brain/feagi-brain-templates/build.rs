use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let generated_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let templates_dir = manifest_dir.join("templates");

    let definitions_rs = templates_dir.join("definitions.rs");
    let cortical_units_rs = templates_dir.join("cortical_units.rs");

    track_template_changes(&definitions_rs, &cortical_units_rs);
}

/// Rebuild when template sources change.
fn track_template_changes(definitions_rs: &Path, cortical_units_rs: &Path) {
    println!("cargo:rerun-if-changed={}", definitions_rs.display());
    println!("cargo:rerun-if-changed={}", cortical_units_rs.display());
    println!("cargo:rerun-if-changed={}", Path::new("build.rs").display());
}

