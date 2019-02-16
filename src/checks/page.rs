use crate::configuration::*;
use crate::products::expected::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Page check structure
pub struct Page {

    /// Page URL
    pub url: String,

    /// Page expectations
    pub expects: Option<PageExpectations>,

    /// Curl options
    pub options: Option<PageOptions>,

}


/// Pages type
pub type Pages = Vec<Page>;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Page options - passed to Curl
pub struct PageOptions {

    /// HTTP cookies
    pub cookies: Option<Vec<String>>,

    /// HTTP headers
    pub headers: Option<Vec<String>>,

    /// HTTP data (body)
    pub data: Option<Vec<String>>,

    /// HTTP follow 301/302 redirects
    pub follow_redirects: Option<bool>,

    /// HTTP method used
    pub method: Option<Method>,

    /// HTTP agent name
    pub agent: Option<String>,

    /// HTTP check timeout in seconds
    pub timeout: Option<u64>,

    /// HTTP connection timeout in seconds
    pub connection_timeout: Option<u64>,

}


impl Default for PageOptions {
    fn default() -> PageOptions {
        PageOptions {
            agent: None,
            cookies: None,
            headers: None,
            data: None,
            follow_redirects: Some(true),
            method: Some(Method::default()),
            timeout: Some(CHECK_TIMEOUT),
            connection_timeout: Some(CHECK_CONNECTION_TIMEOUT),
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
