use std::io::Error;


/// Checks trait
pub trait Checks<T> {

    /// Load check from any source
    fn load(name: &str) -> Result<T, Error>;

}
