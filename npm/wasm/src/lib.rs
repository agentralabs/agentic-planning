// AgenticPlanning WASM bindings

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn default_file_name() -> String {
    "project.aplan".to_string()
}
