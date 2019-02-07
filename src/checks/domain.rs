use crate::products::expected::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Domain check structure
pub struct Domain {

    /// Domain name
    pub name: String,

    /// Domain expectations
    pub expects: Option<DomainExpectations>,

}


/// Domains type
pub type Domains = Vec<Domain>;
