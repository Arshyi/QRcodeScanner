//! Native QRForge desktop entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    qrforge_desktop_lib::run();
}
