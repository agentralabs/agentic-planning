//! Query layer — intent scoping, delta retrieval, token budgets, pagination.

pub mod intent;
pub mod delta;
pub mod budget;
pub mod pagination;

pub use intent::*;
pub use delta::*;
pub use budget::*;
pub use pagination::*;
