//! Contract surface for AgenticPlanning.
//!
//! This module keeps canonical parity with sister crates that expose
//! a `contracts` module backed by shared SDK concepts.

use serde_json::{json, Value};

/// Minimal contract response shape used by MCP/CLI call paths.
pub fn sister_contract_info() -> Value {
    json!({
        "name": "AgenticPlanning",
        "key": "planning",
        "format": ".aplan",
        "status": "active"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_info_has_expected_shape() {
        let info = sister_contract_info();
        assert_eq!(info["name"], "AgenticPlanning");
        assert_eq!(info["key"], "planning");
        assert_eq!(info["format"], ".aplan");
    }
}
