use crate::*;
use curl::{multi::Easy2Handle, MultiError};


/// Type alias for long type name:
pub type CurlHandler = Result<Easy2Handle<Collector>, MultiError>;
