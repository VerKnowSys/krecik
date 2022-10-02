use crate::*;


#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq, Hash)]
/// Unexpected check result that's considered minor - we don't want notifications from those
pub enum UnexpectedMinor {
    /// HttpCode (url, code)
    #[error("URL: \"{0}\" unavailable. Details: {1}")]
    OSError(String, String),

    /// Failed internal function
    #[error("InternalProtocolProblemFailure on: {0}. Details: {1}")]
    InternalProtocolProblem(String, String),

    /// Curl multi handler minor failure
    #[error("{0}")]
    HandlerFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq, Eq, Hash)]
/// Unexpected check result
pub enum Unexpected {
    /// Failed to pass page expectation
    #[error("Failed to connect to URL: \"{0}\". Details: {1}")]
    URLConnectionProblem(String, String),

    /// Failed to pass page expectation
    #[error("Expired SSL certificate for domain: \"{0}\", which is valid only for: {1} days")]
    TLSDomainExpired(String, i32),

    /// Failed to get expected Address
    #[error("URL: \"{0}\" has invalid final address: \"{1}\". Expected: \"{2}\"")]
    AddressInvalid(String, String, String),

    /// Curl multi handler failure
    #[error("{0}")]
    HandlerFailed(String),

    /// Http connection failed
    #[error("URL: \"{0}\" couldn't be reached in time frame of {1} seconds")]
    HttpConnectionFailed(String, u64),

    /// HttpCode (url, code)
    #[error("URL: \"{0}\" returned error: {1}. Expected code: {2}")]
    HttpCodeInvalid(String, u32, u32),

    /// Content - expected content not found where expected
    #[error("URL: \"{0}\" lacks expected content: \"{1}\"")]
    ContentInvalid(String, String),

    /// Failed content length check
    #[error(
        "URL: \"{0}\" is unable to pass minimum-content-length expectation! Actual content length: {1}. Expected minimum-length: {2}"
    )]
    ContentLengthInvalid(String, usize, usize),

    /// Check file parse error
    #[error("Failed to parse check input data! Error details: \"{0}\"")]
    CheckParseProblem(String),

    /// Not Implemented functionality
    #[error("Unmatched validation case for: \"{0}\". Details: {1}")]
    UnmatchedValidationCase(String, String),
}
