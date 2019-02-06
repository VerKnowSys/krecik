

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Describes all supported page expectations
pub enum PageExpectation {

    /// Valid error code
    ValidCode (i64),

    /// Valid content regex match
    ValidContent (String),

    /// Valid address regex match
    ValidAddress (String),

    /// Valid content length
    ValidLength (u64),

}


impl Default for PageExpectation {
    fn default() -> PageExpectation {
        PageExpectation::ValidCode(200)
    }
}


/// Page expectations type
pub type PageExpectations = Vec<PageExpectation>;


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
/// Describes all supported domain expectations
pub enum DomainExpectation {

    /// Domain is resolvable
    ValidResolvable,

    /// Domain expiry minimum period in days
    ValidExpiryPeriod (u64),

}


impl Default for DomainExpectation {
    fn default() -> DomainExpectation {
        DomainExpectation::ValidResolvable
    }
}

/// Domain expectations type
pub type DomainExpectations = Vec<DomainExpectation>;
