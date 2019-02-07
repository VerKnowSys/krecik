use std::io::Error;

use crate::checks::domain::*;
use crate::checks::page::*;
use crate::products::expected::*;
use crate::products::unexpected::*;


/// generic Check, specialized via defined inputs
pub trait Check<T> {

    /// Load check from json
    fn load(name: &str) -> Result<T, Error>;

}
