use std::io::Error;


/// Checks trait
pub trait Checks<T> {

    /// Load check from any source
    fn load(name: &str) -> Result<T, Error>;

    /// Load check from file in non standard dir
    fn load_from(name: &str, checks_dir: &str) -> Result<T, Error>;

}
