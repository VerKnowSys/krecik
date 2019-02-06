use crate::products::expected::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Unexpected check result
pub enum Unexpected {

    /// Failed to pass domain expectation
    FailedDomain (String, DomainExpectation),

    /// Failed to pass page expectation
    FailedPage (String, PageExpectation),

    /// Failed to read/parse JSON
    FailedJson (String),

    /// Failed to connect/read from remote
    FailedRemote (String),

}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Story holds errornous state
pub struct Story {

    /// Story - timestamp
    timestamp: u64,

    /// Story - failure count
    count: u64,

    /// Story - keep history of unexpected results
    error: Unexpected,

}


/// History type
pub type History = Vec<Story>;
