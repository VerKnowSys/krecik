//! "Traversing Mole" utility: Krecik

//! Crate docs

#![forbid(unsafe_code)]
#![deny(
    unstable_features,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
#![warn(dead_code, unused_imports, unused_variables)]

// For development:
// #![allow(dead_code, unused_imports, unused_variables, deprecated)]


/// Use MiMalloc as default allocator:
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;


// Load all useful macros:

pub use crate::actors::notificator::*;
pub use crate::api::*;
pub use crate::checks::check::*;
pub use crate::checks::page::*;
pub use crate::checks::pongo::*;
pub use crate::config::*;
pub use crate::configuration::*;
pub use crate::products::expected::*;
pub use crate::products::history::*;
pub use crate::products::story::*;
pub use crate::products::unexpected::*;
pub use crate::utilities::*;
use core::fmt::{Debug, Formatter};
use curl::easy::{Handler, WriteError};
pub use serde::{Deserialize, Serialize};
use std::fmt;
pub use thiserror::Error;
pub use tracing::{debug, error, event, info, instrument, span, trace, warn, Level};


/// Collects async content from Curl:
pub struct Collector(Vec<u8>);


impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}


impl Debug for Collector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!(
            "Collector buffer (first 50 in hex): {}…",
            self.to_string()
        ))
        .finish()
    }
}


impl ToString for Collector {
    fn to_string(&self) -> String {
        self.0.iter().take(50).map(|c| format!("{:x}", c)).collect()
    }
}


//
// Public modules:
//

/// Configuration defaults:
pub mod configuration;

/// Dynamic config:
pub mod config;

/// Utilities and helpers:
pub mod utilities;

/// Checks:
pub mod checks;

/// Check products:
pub mod products;

/// Checks API functions:
pub mod api;

/// Actors:
pub mod actors;

//
// Private modules:
//

/// Tests:
#[cfg(test)]
mod tests;
