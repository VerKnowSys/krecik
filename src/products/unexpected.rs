#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq, Eq, Hash)]
/// Unexpected check result
pub enum Unexpected {
    /// Failed to pass page expectation
    #[fail(display = "Failed to connect to URL: {}. Details: {}.", _0, _1)]
    URLConnectionProblem(String, String),

    /// Failed to pass page expectation
    #[fail(display = "Expired TLS/SSL Certificate for domain: {}.", _0)]
    TLSDomainExpired(String),

    /// Failed to get expected Address
    #[fail(
        display = "URL: {} has invalid final address: \"{}\". Expected: \"{}\"",
        _0, _1, _2
    )]
    AddressInvalid(String, String, String),

    /// Curl multi handler failure
    #[fail(display = "Curl handler failure: {}.", _0)]
    HandlerFailed(String),
    /// HttpCode (url, code)
    #[fail(
        display = "URL: {} responded with unexpected error-code: {}. Expected code: {}",
        _0, _1, _2
    )]
    HttpCodeInvalid(String, u32, u32),

    /// Content - expected content not found where expected
    #[fail(display = "URL: {} response lacks expected content: \"{}\"", _0, _1)]
    ContentInvalid(String, String),

    /// Failed content length check
    #[fail(
        display = "URL: {} is unable to pass minimum-content-length expectation! Actual content length: {}. Expected minimum-length: {}.",
        _0, _1, _2
    )]
    ContentLengthInvalid(String, usize, usize),

    /// Failed internal function
    #[fail(
        display = "InternalProtocolProblemFailure on: {}. Details: {}.",
        _0, _1
    )]
    InternalProtocolProblem(String, String),

    /// Check file parse error
    #[fail(
        display = "Failed to parse check input data! Error details: \"{}\".",
        _0
    )]
    CheckParseProblem(String),

    /// Not Implemented functionality
    #[fail(
        display = "Unmatched validation case for: \"{}\". Details: {}.",
        _0, _1
    )]
    UnmatchedValidationCase(String, String),
}
