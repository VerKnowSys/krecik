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

    /// Failed to pass domain expectation
    #[fail(display = "Domain expectation: {} has failed", _0)]
    FailedDomain (String),

    /// Failed to pass page expectation
    #[fail(display = "Page expectation has failed for page: {}", _0)]
    FailedPage (String),

    /// Failed to read/parse JSON
    #[fail(display = "JSON parse failure for: {}", _0)]
    FailedJson (String),

    /// Failed to connect/read from remote
    #[fail(display = "Remote access failure for: {}", _0)]
    FailedRemote (String),

    /// Failed content check
    #[fail(display = "Content validation failure for: {}", _0)]
    FailedContent (String),

    /// Failed internal function
    #[fail(display = "Internal failure for: {}", _0)]
    FailedInternal (String),

}
