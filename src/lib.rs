//! "Traversing Mole" utility

//! Crate docs

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

#![allow(dead_code)]

/// Use Jemalloc as default allocator:
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;


// Load all useful macros:

#[allow(unused_imports)]
#[macro_use]
pub extern crate log;

#[allow(unused_imports)]
#[macro_use]
pub extern crate failure;

#[allow(unused_imports)]
#[macro_use]
pub extern crate serde_derive;


//
// Public modules:
//

/// Configuration defaults:
pub mod configuration;

/// Checks:
pub mod checks;

/// Resource inputs:
pub mod inputs;

/// Check products:
pub mod products;

/// Web router:
pub mod webrouter;


//
// Private modules:
//

/// Tests:
mod tests;
