#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq, Eq, Hash)]
/// Unexpected check result that's considered minor - we don't want notifications from those
pub enum UnexpectedMinor {
    /// HttpCode (url, code)
    #[fail(display = "URL: {} unavailable. Details: {}", _0, _1)]
    OSError(String, String),

    /// Failed internal function
    #[fail(display = "InternalProtocolProblemFailure on: {}. Details: {}", _0, _1)]
    InternalProtocolProblem(String, String),

    /// Curl multi handler minor failure
    #[fail(display = "{}", _0)]
    HandlerFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq, Eq, Hash)]
/// Unexpected check result
pub enum Unexpected {
    /// Failed to pass page expectation
    #[fail(display = "Failed to connect to URL: {}. Details: {}", _0, _1)]
    URLConnectionProblem(String, String),

    /// Failed to pass page expectation
    #[fail(
        display = "Expired SSL certificate for domain: {}, which is valid only for: {} days",
        _0, _1
    )]
    TLSDomainExpired(String, i32),

    /// Failed to get expected Address
    #[fail(
        display = "URL: {} has invalid final address: \"{}\". Expected: \"{}\"",
        _0, _1, _2
    )]
    AddressInvalid(String, String, String),

    /// Curl multi handler failure
    #[fail(display = "{}", _0)]
    HandlerFailed(String),

    /// Http connection failed
    #[fail(
        display = "URL: {} couldn't be reached in time frame of {} seconds",
        _0, _1
    )]
    HttpConnectionFailed(String, u64),

    /// HttpCode (url, code)
    #[fail(display = "URL: {} returned error: {}. Expected code: {}", _0, _1, _2)]
    HttpCodeInvalid(String, u32, u32),

    /// Content - expected content not found where expected
    #[fail(display = "URL: {} lacks expected content: \"{}\"", _0, _1)]
    ContentInvalid(String, String),

    /// Failed content length check
    #[fail(
        display = "URL: {} is unable to pass minimum-content-length expectation! Actual content length: {}. Expected minimum-length: {}",
        _0, _1, _2
    )]
    ContentLengthInvalid(String, usize, usize),

    /// Check file parse error
    #[fail(
        display = "Failed to parse check input data! Error details: \"{}\"",
        _0
    )]
    CheckParseProblem(String),

    /// Not Implemented functionality
    #[fail(display = "Unmatched validation case for: \"{}\". Details: {}", _0, _1)]
    UnmatchedValidationCase(String, String),
}
