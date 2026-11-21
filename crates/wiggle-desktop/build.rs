fn main() {
    // tauri-winres shells out to windres, which cannot handle spaces in
    // the workspace path (it splits "Default Project" in two). Skip the
    // resource-embedding step on such paths; CI release bundles build
    // from space-free checkouts where the full step runs.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if manifest.contains(' ') {
        println!(
            "cargo:warning=skipping tauri-build resource step: workspace path contains spaces ({manifest})"
        );
    } else {
        tauri_build::build();
    }
}
