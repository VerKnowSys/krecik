use crate::checks::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Page check structure
pub struct Page {
    /// Page URL
    pub url: String,

    /// Page expectations
    #[serde(default = "default_page_expectations")]
    pub expects: PageExpectations,

    /// Curl options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PageOptions>,
}


/// Provide own default page expectations if nothing defined in check input:
pub fn default_page_expectations() -> PageExpectations {
    vec![
        PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE),
        PageExpectation::ValidLength(CHECK_HTTP_MINIMUM_LENGHT),
        PageExpectation::ValidContent(CHECK_DEFAULT_CONTENT_EXPECTATION.to_string()),
    ]
}


/// Pages type
pub type Pages = Vec<Page>;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Page options - passed to Curl
pub struct PageOptions {
    /// HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<String>>,

    /// HTTP POST data (body)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_data: Option<Vec<String>>,

    /// HTTP cookies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<String>>,

    /// HTTP follow 301/302 redirects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,

    /// HTTP method used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<Method>,

    /// HTTP agent name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// HTTP check timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// HTTP connection timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_timeout: Option<u64>,

    /// HTTP connection timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,

    /// TLS peer verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl_verify_peer: Option<bool>,

    /// TLS host verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl_verify_host: Option<bool>,
}


/// Implement JSON serialization on .to_string():
impl ToString for PageOptions {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| {
            String::from("{\"status\": \"PageOptions serialization failure\"}")
        })
    }
}


impl Default for PageOptions {
    fn default() -> PageOptions {
        PageOptions {
            method: Some(Method::default()), // GET
            timeout: Some(CHECK_TIMEOUT),
            connection_timeout: Some(CHECK_CONNECTION_TIMEOUT),
            ssl_verify_peer: Some(true),
            ssl_verify_host: Some(true),
            follow_redirects: Some(true),

            agent: None,
            cookies: None,
            headers: None,
            post_data: None,
            verbose: None,
        }
    }
}


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
/// HTTP methods allowed
pub enum Method {
    /// HTTP HEAD
    HEAD,

    /// HTTP PUT
    PUT,

    /// HTTP GET
    GET,

    /// HTTP POST
    POST,

    /// HTTP DELETE
    DELETE,
}


impl Default for Method {
    fn default() -> Method {
        Method::GET
    }
}
