use crate::*;


#[derive(
    Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
/// Describes all supported page expectations
pub enum PageExpectation {
    /// Valid error code
    #[error("ValidCode: {0}.")]
    ValidCode(u32),

    /// Valid no-content check
    #[error("ValidNoContent.")]
    ValidNoContent,

    /// Valid content regex match
    #[error("ValidContent: {0}.")]
    ValidContent(String),

    /// Valid content length
    #[error("ValidLength: {0} bytes.")]
    ValidLength(usize),

    /// Valid no-content-length check
    #[error("ValidNoLength.")]
    ValidNoLength,

    /// Valid final address (after all redirections)
    #[error("ValidAddress: {0}")]
    ValidAddress(String),

    /// Valid no-address check
    #[error("ValidNoAddress.")]
    ValidNoAddress,
}


/// Page expectations type
pub type PageExpectations = Vec<PageExpectation>;


#[derive(
    Debug, Copy, Clone, Serialize, Deserialize, Error, PartialEq, Eq, PartialOrd, Ord,
)]
/// Describes all supported domain expectations
pub enum DomainExpectation {
    /// Domain expiry minimum period in days
    #[error("ValidExpiryPeriod: {0} days.")]
    ValidExpiryPeriod(i32),
}


/// Domain expectations type
pub type DomainExpectations = Vec<DomainExpectation>;


#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq, Hash)]
/// All response types for all supported expectations
pub enum Expected {
    /// Check returned expected Address
    #[error("URL: \"{0}\" returns expected final-address: \"{1}\".")]
    Address(String, String),

    /// Check returned expected HTTP error code
    #[error("URL: \"{0}\" returns expected error-code: {1}.")]
    HttpCode(String, u32),

    /// Check returned expected page contents
    #[error("URL: \"{0}\" contains expected literal: \"{1}\".")]
    Content(String, String),

    /// NoContentLength
    #[error("URL: \"{0}\" no content-length validation.")]
    NoContentLength(String),

    /// EmptyContent
    #[error("URL: \"{0}\" no content validation.")]
    EmptyContent(String),

    /// Check returned expected page content length
    #[error("URL: \"{0}\" has minimum content-length at least: {1} bytes long.")]
    ContentLength(String, usize),

    /// Check TLS certificate expiration time
    #[error(
        "TLS certificate for domain: \"{0}\", will be valid for: {1} more days. Requested minimum: {2} days."
    )]
    TLSCertificateFresh(String, i32, i32),
}
