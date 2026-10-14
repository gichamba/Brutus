//! Executable entry point for the Brutus desktop shell.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Start the desktop app without a console window in Windows release builds.
fn main() {
    brutus_desktop::run();
}
