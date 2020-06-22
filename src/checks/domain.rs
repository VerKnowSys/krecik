use crate::configuration::*;
use crate::products::expected::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Domain check structure
pub struct Domain {
    /// Domain name
    pub name: String,

    /// Domain expectations
    #[serde(default = "default_domain_expectations")]
    pub expects: DomainExpectations,
}


/// Domains type
pub type Domains = Vec<Domain>;


/// Provide own default domain expectations if nothing defined in check input:
pub fn default_domain_expectations() -> DomainExpectations {
    vec![DomainExpectation::ValidExpiryPeriod(
        CHECK_MINIMUM_DAYS_OF_TLSCERT_VALIDITY,
    )]
}
