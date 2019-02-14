use failure::Error;

use crate::products::unexpected::*;


#[derive(Debug, Clone, Serialize, Deserialize, Fail)]
/// Describes all supported page expectations
pub enum PageExpectation {

    /// Valid error code
    #[fail(display = "Passed ValidCode: '{}'", _0)]
    ValidCode (i64),

    /// Valid content regex match
    #[fail(display = "Passed ValidContent: '{}'", _0)]
    ValidContent (String),

    /// Valid address regex match
    #[fail(display = "Passed ValidAddress: '{}'", _0)]
    ValidAddress (String),

    /// Valid content length
    #[fail(display = "Passed ValidLength: '{}'", _0)]
    ValidLength (u64),

}


impl Default for PageExpectation {
    fn default() -> PageExpectation {
        PageExpectation::ValidCode(200)
    }
}


/// Page expectations type
pub type PageExpectations = Vec<PageExpectation>;


#[derive(Debug, Copy, Clone, Serialize, Deserialize, Fail)]
/// Describes all supported domain expectations
pub enum DomainExpectation {

    /// Domain is resolvable
    #[fail(display = "Passed ValidResolvable")]
    ValidResolvable,

    /// Domain expiry minimum period in days
    #[fail(display = "Passed ValidExpiryPeriod: '{}'", _0)]
    ValidExpiryPeriod (u64),

}


impl Default for DomainExpectation {
    fn default() -> DomainExpectation {
        DomainExpectation::ValidResolvable
    }
}


/// Domain expectations type
pub type DomainExpectations = Vec<DomainExpectation>;
