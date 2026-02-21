//! Shared data models for vga-swarm.
//!
//! Types are grouped by domain into submodules; everything is re-exported at
//! this level so existing `use crate::shared::models::*` imports continue to work.

pub mod core;
pub mod network;
pub mod resource;
pub mod vault;

pub use core::*;
pub use network::*;
pub use resource::*;
pub use vault::*;
