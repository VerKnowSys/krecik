use crate::products::expected::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Domain check structure
pub struct Domain {

    /// Domain name
    pub name: String,

    /// Domain expectations
    #[serde(skip_serializing_if = "Option::is_none", default = "default_domain_expectations")]
    pub expects: Option<DomainExpectations>,

}


/// Domains type
pub type Domains = Vec<Domain>;


/// Provide own default domain expectations if nothing defined in check input:
fn default_domain_expectations() -> Option<DomainExpectations> {
    Some(
        vec![
            DomainExpectation::ValidExpiryPeriod(14)
        ]
    )
}
