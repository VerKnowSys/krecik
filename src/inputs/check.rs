use std::io::Error;

use crate::products::unexpected::*;


/// Checks trait
pub trait Checks<T> {

    /// Load check from any source
    fn load(name: &str) -> Result<T, Error>;

    /// Execute loaded checks
    fn execute(&self) -> Result<(), History>;

    /// Check domains
    fn check_domains(&self) -> Result<(), History>;

    /// Check pages
    fn check_pages(&self) -> Result<(), History>;

}
