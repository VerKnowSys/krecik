use crate::products::expected::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Domain check structure
pub struct Domain {

    /// Domain name
    name: String,

    /// Domain expectations
    expects: DomainExpectations,

}


/// Domains type
pub type Domains = Vec<Domain>;
