use hyper::*;
use gotham::helpers::http::response::create_response;
use gotham::state::State;
use mime::APPLICATION_JSON;
use gotham::handler::IntoResponse;
use chrono::Local;
use failure::Error;

use crate::products::expected::*;


#[derive(Debug, Clone, Serialize, Deserialize, Fail, PartialEq)]
/// Unexpected check result
pub enum Unexpected {

    /// Failed to pass page expectation
    #[fail(display = "Failed to connect to URL: {}. Details: {}.", _0, _1)]
    URLConnectionProblem (String, String),

    /// Failed to pass page expectation
    #[fail(display = "Expired TLS/SSL Certificate for domain: {}.", _0)]
    TLSDomainExpired (String),

    /// HttpErrorCode (url, got_code, expected_code)
    #[fail(display = "URL: {} responded with unexpected error-code: {} but code: {} was expected.", _0, _1, _2)]
    HttpErrorCode (String, u32, u32),

    /// Failed content length check
    #[fail(display = "URL: {} is unable to pass minimum-content-length expectation! Actual content length: {}. Expected minimum-length: {}.", _0, _1, _2)]
    MinimumContentLength (String, usize, usize),

    /// Failed internal function
    #[fail(display = "InternalProtocolProblemFailure on: {}. Details: {}.", _0, _1)]
    InternalProtocolProblem (String, String),

    /// EmptyContent
    #[fail(display = "URL: {} responded with empty content.", _0)]
    EmptyContent (String),

    /// ZeroLengthContent
    #[fail(display = "URL: {} responded with 0 content-length.", _0)]
    ZeroLengthContent (String),

    /// InvalidContent - expected content not found where expected
    #[fail(display = "URL: {} responded with invalid content: {}", _0, _1)]
    InvalidContent (String, String),

    /// Not Implemented functionality
    #[fail(display = "Not Implemented yet: {}. Details: {}.", _0, _1)]
    NotImplementedYet (String, String),

}
