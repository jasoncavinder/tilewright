// SPDX-License-Identifier: MPL-2.0

//! Core domain types and operations for Tilewright.
//!
//! This crate should remain independent of MCP, AI models, user interfaces,
//! and any particular agent host.

/// Current Tilewright library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
