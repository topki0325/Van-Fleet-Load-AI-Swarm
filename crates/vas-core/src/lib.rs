// Core crate that reuses the existing source tree for backend/shared.

#[path = "../../../src/shared/mod.rs"]
pub mod shared;

#[path = "../../../src/backend/mod.rs"]
pub mod backend;
