use failure::Error;

use crate::configuration::*;
use crate::products::unexpected::*;


#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq)]
/// Describes all supported page expectations
pub enum PageExpectation {

    /// Valid error code
    #[fail(display = "ValidCode: {}.", _0)]
    ValidCode (u32),

    /// Valid content regex match
    #[fail(display = "ValidContent: {}.", _0)]
    ValidContent (String),

    /// Valid content length
    #[fail(display = "ValidLength: {} bytes.", _0)]
    ValidLength (usize),

    /// Valid final address (after all redirections)
    #[fail(display = "ValidAddress: {}", _0)]
    ValidAddress (String),

}


impl Default for PageExpectation {
    fn default() -> PageExpectation {
        PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE)
    }
}


/// Page expectations type
pub type PageExpectations = Vec<PageExpectation>;


#[derive(Debug, Copy, Clone, Serialize, Deserialize, Fail)]
/// Describes all supported domain expectations
pub enum DomainExpectation {

    /// Domain expiry minimum period in days
    #[fail(display = "ValidExpiryPeriod: {} days.", _0)]
    ValidExpiryPeriod (i32),

}


impl Default for DomainExpectation {
    fn default() -> DomainExpectation {
        DomainExpectation::ValidExpiryPeriod(CHECK_SSL_DAYS_EXPIRATION)
    }
}


/// Domain expectations type
pub type DomainExpectations = Vec<DomainExpectation>;



#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq)]
/// All response types for all supported expectations
pub enum Expected {

    /// Check returned expected Address
    #[fail(display = "URL: {} returned expected final address: {}.", _0, _1)]
    Address (String, String),


    /// Check returned expected HTTP error code
    #[fail(display = "URL: {} returned expected error-code: {}.", _0, _1)]
    HttpCode (String, u32),


    /// Check returned expected page contents
    #[fail(display = "URL: {} contains expected value: \"{}\".", _0, _1)]
    Content (String, String),


    /// NoContentLength
    #[fail(display = "URL: {} no content-length validation.", _0)]
    NoContentLength (String),


    /// EmptyContent
    #[fail(display = "URL: {} no content validation.", _0)]
    EmptyContent (String),


    /// Check returned expected page content length
    #[fail(display = "URL: {} returned expected content-length at least: {} bytes long.", _0, _1)]
    ContentLength (String, usize),


    /// Check TLS certificate expiration time
    #[fail(display = "TLS certificate for domain: {}, will be valid for: {} more days. Requested minimum: {} days.", _0, _1, _2)]
    TLSCertificateFresh (String, i32, i32),


}



