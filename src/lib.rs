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
// For development:
#![allow(dead_code, unused_imports, unused_variables)]


/// Use Jemalloc as default allocator:
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;


// Load all useful macros:

#[macro_use]
pub extern crate log;

#[macro_use]
pub extern crate failure;

#[macro_use]
pub extern crate serde_derive;


use crate::checks::domain::*;
use crate::checks::generic::*;
use crate::checks::page::*;
use crate::checks::pongo::*;
use crate::checks::*;
use crate::configuration::*;
use crate::products::expected::*;
use crate::products::history::*;
use crate::products::story::*;
use crate::products::unexpected::*;
use crate::utilities::*;


//
// Public modules:
//

/// Configuration defaults:
pub mod configuration;

/// Utilities and helpers:
pub mod utilities;

/// Checks:
pub mod checks;

/// Check products:
pub mod products;

/// Checks API functions:
pub mod api;


//
// Private modules:
//

/// Tests:
mod tests;
