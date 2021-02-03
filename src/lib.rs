//! "Traversing Mole" utility

//! Crate docs

#![forbid(unsafe_code)]
#![deny(
    missing_docs,
    unstable_features,
    unsafe_code,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![warn(dead_code, unused_imports, unused_variables)]
// For development:
#![allow(deprecated)]
// #![allow(dead_code, unused_imports, unused_variables, deprecated)]


/// Use MiMalloc as default allocator:
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;


// Load all useful macros:

#[macro_use]
pub extern crate log;

#[macro_use]
pub extern crate failure;

#[macro_use]
pub extern crate serde_derive;


pub use crate::actors::notificator::*;
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
use curl::easy::{Handler, WriteError};


/// Collects async content from Curl:
#[derive(Debug)]
pub struct Collector(pub Vec<u8>);


impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
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
mod tests;
