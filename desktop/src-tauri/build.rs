//! Generate Tauri's build metadata and platform resources.

/// Run Tauri's build-time configuration and resource checks.
fn main() {
    tauri_build::build();
}
