//! Shared data models for ollama-lan-share.
//!
//! Types are grouped by domain into submodules; everything is re-exported at
//! this level so existing `use crate::shared::models::*` imports continue to work.

pub mod core;
pub mod network;

pub use core::*;
pub use network::*;