#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq, Eq, Hash)]
/// Describes all supported page expectations
pub enum PageExpectation {
    /// Valid error code
    #[fail(display = "ValidCode: {}.", _0)]
    ValidCode(u32),

    /// Valid no-content check
    #[fail(display = "ValidNoContent.")]
    ValidNoContent,

    /// Valid content regex match
    #[fail(display = "ValidContent: {}.", _0)]
    ValidContent(String),

    /// Valid content length
    #[fail(display = "ValidLength: {} bytes.", _0)]
    ValidLength(usize),

    /// Valid no-content-length check
    #[fail(display = "ValidNoLength.")]
    ValidNoLength,

    /// Valid final address (after all redirections)
    #[fail(display = "ValidAddress: {}", _0)]
    ValidAddress(String),

    /// Valid no-address check
    #[fail(display = "ValidNoAddress.")]
    ValidNoAddress,
}


/// Page expectations type
pub type PageExpectations = Vec<PageExpectation>;


#[derive(Debug, Copy, Clone, Serialize, Deserialize, Fail)]
/// Describes all supported domain expectations
pub enum DomainExpectation {
    /// Domain expiry minimum period in days
    #[fail(display = "ValidExpiryPeriod: {} days.", _0)]
    ValidExpiryPeriod(i32),
}


/// Domain expectations type
pub type DomainExpectations = Vec<DomainExpectation>;


#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq, Eq, Hash)]
/// All response types for all supported expectations
pub enum Expected {
    /// Check returned expected Address
    #[fail(display = "URL: {} returns expected final-address: \"{}\".", _0, _1)]
    Address(String, String),

    /// Check returned expected HTTP error code
    #[fail(
        display = "URL: {} returns expected error-code: {}. Took: {} ms.",
        _0, _1, _2
    )]
    HttpCode(String, u32, u128),

    /// Check returned expected page contents
    #[fail(display = "URL: {} contains expected literal: \"{}\".", _0, _1)]
    Content(String, String),

    /// NoContentLength
    #[fail(display = "URL: {} no content-length validation.", _0)]
    NoContentLength(String),

    /// EmptyContent
    #[fail(display = "URL: {} no content validation.", _0)]
    EmptyContent(String),

    /// Check returned expected page content length
    #[fail(
        display = "URL: {} has minimum content-length at least: {} bytes long.",
        _0, _1
    )]
    ContentLength(String, usize),

    /// Check TLS certificate expiration time
    #[fail(
        display = "TLS certificate for domain: {}, will be valid for: {} more days. Requested minimum: {} days.",
        _0, _1, _2
    )]
    TLSCertificateFresh(String, i32, i32),
}
