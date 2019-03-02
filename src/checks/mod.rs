/// Domain checks:
pub mod domain;

/// Page checks:
pub mod page;

/// JSON generic Check:
pub mod generic;

/// Mapper for default remote Checks lilst JSON resource: Pongo
pub mod pongo;


use curl::MultiError;
use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, List, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;
use colored::Colorize;
use rayon::prelude::*;

use crate::configuration::*;
use crate::utilities::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::products::history::*;


/// Collects async content from Curl:
#[derive(Debug)]
pub struct Collector(Vec<u8>);


impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

/// Type alias for long type name:
pub type CurlHandler = Result<Easy2Handle<Collector>, MultiError>;


/// Checks trait
pub trait Checks<T> {


    /// Load check from any source
    fn load(name: &str) -> Result<T, Error>;


    /// Execute loaded checks
    fn execute(&self) -> History;


    /// Check SSL certificate expiration using OpenSSL function
    fn check_ssl_expire(domain_name: &str, domain_expectation: DomainExpectation) -> Story {
         SslExpiration::from_domain_name(&domain_name)
             .and_then(|ssl_validator|
                 match domain_expectation {
                    DomainExpectation::ValidExpiryPeriod(expected_days) =>
                         if ssl_validator.days() < expected_days
                         || ssl_validator.is_expired() {
                            Ok(Story::new_error(Some(Unexpected::TLSDomainExpired(domain_name.to_string()))))
                         } else {
                            Ok(Story::new(Some(Expected::TLSCertificateFresh(domain_name.to_string(), ssl_validator.days(), expected_days))))
                         }
                 }
             )
             .unwrap_or_else(|err| Story::new_error(Some(Unexpected::InternalProtocolProblem(domain_name.to_string(), err.0.to_string()))))
    }


    /// Check domains
    fn check_domains(domains: Option<Domains>) -> History {
        domains
            .and_then(|domains|
                Some(
                    History::new_from(
                        domains
                            .into_par_iter()
                            .flat_map(|defined_check| {
                                let domain_check = defined_check.clone();
                                let domain_name = domain_check.name;
                                let domain_expectations
                                    = domain_check
                                        .expects
                                        .unwrap_or_else(Self::default_domain_expectations);
                                debug!("check_domain::domain_expectations: {}", format!("{:?}", domain_expectations).magenta());

                                // Process Domain expectations using parallel iterator (Rayon):
                                History::new_from(
                                    domain_expectations
                                        .into_par_iter()
                                        .map(|domain_expectation| {
                                            let domain_story = Self::check_ssl_expire(&domain_name, domain_expectation);
                                            match domain_story.clone() {
                                                Story{ success: Some(success_msg), error: None, .. } =>
                                                    info!("Domain TLS-cert validity Story of: SUCCESS: {}", success_msg.to_string().green()),

                                                Story{ success: None, error: Some(error_msg), .. } =>
                                                    error!("Domain TLS-cert validity Story of: FAILURE: {}", error_msg.to_string().red()),

                                                Story{ success: None, error: None, .. } =>
                                                    warn!("Domain TLS-cert validity Story of: WARNING: {}", "Ambiguous Story that lacks both success and error?!".yellow()),

                                                Story{ success: Some(_), error: Some(_), .. } =>
                                                    warn!("Domain TLS-cert validity Story of: WARNING: {}", "Ambiguous Story with success and failure at the same time?!".yellow()),
                                            };
                                            domain_story
                                        })
                                        .collect()
                                ).stories()
                            })
                            .collect()
                        )
                )
            )
            .unwrap_or_else(History::empty)
    }


    /// Find and extract code validation from validations
    fn find_code_validation(page_expectations: &[PageExpectation]) -> &PageExpectation {
        page_expectations
            .par_iter()
            .find_any(|exp| {
                match exp {
                    PageExpectation::ValidCode(_) => true,
                    _ => false
                }
            })
            .unwrap_or_else(|| &PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE))
    }


    /// Find and extract content validation from validations
    fn find_content_validation(page_expectations: &[PageExpectation]) -> &PageExpectation {
        page_expectations
            .par_iter()
            .find_any(|exp| {
                match exp {
                    PageExpectation::ValidContent(_) => true,
                    _ => false,
                }
            })
            .unwrap_or_else(|| &PageExpectation::ValidNoContent)
    }


    /// Find and extract content length validation from validations
    fn find_content_length_validation(page_expectations: &[PageExpectation]) -> &PageExpectation {
        page_expectations
            .par_iter()
            .find_any(|exp| {
                match exp {
                    PageExpectation::ValidLength(_) => true,
                    _ => false,
                }
            })
            .unwrap_or_else(|| &PageExpectation::ValidNoLength)
    }


    /// Find and extract address validation from validations
    fn find_address_validation(page_expectations: &[PageExpectation]) -> &PageExpectation {
        page_expectations
            .par_iter()
            .find_any(|exp| {
                match exp {
                    PageExpectation::ValidAddress(_) => true,
                    _ => false,
                }
            })
            .unwrap_or_else(|| &PageExpectation::ValidNoAddress)
    }


    /// Build headers List for Curl
    fn list_of_headers(headers: Option<Vec<String>>) -> List {
        // Build List of HTTP headers
        // ex. header:
        //         list.append("Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==").unwrap();
        let mut list = List::new();
        for header in headers.unwrap_or_default() {
            debug!("Setting Curl header: {}", header.magenta());
            list
                .append(&header.to_owned())
                .unwrap_or_default();
        };
        list
    }


    /// Provide own default page expectations if nothing defined in check input:
    fn default_page_expectations() -> PageExpectations {
        vec![
            PageExpectation::ValidCode(200),
            PageExpectation::ValidLength(100),
            PageExpectation::ValidContent("<body".to_string()),
        ]
    }


    /// Provide own default domain expectations if nothing defined in check input:
    fn default_domain_expectations() -> DomainExpectations {
        vec![
            DomainExpectation::ValidExpiryPeriod(14)
        ]
    }


    /// Load page check handler
    fn load_handler_for(page_check: &Page, multi: &Multi) -> CurlHandler {
        // Initialize Curl, set URL
        let mut curl = Easy2::new(Collector(Vec::new()));
        curl
            .url(&page_check.url)
            .unwrap_or_default();
        debug!("Curl URL:: {}", format!("{}", &page_check.url.magenta()));

        // Load Curl request options from check:
        let curl_options
            = page_check
                .clone()
                .options
                .unwrap_or_default();
        debug!("Curl options: {}", format!("{}", curl_options.to_string().magenta()));

        // Setup Curl configuration based on given options
        if curl_options.follow_redirects.unwrap_or_else(|| true) {
            debug!("Enabled following redirects.");
            curl.follow_location(true).unwrap_or_default();
        } else {
            debug!("Disabled following redirects.");
            curl.follow_location(false).unwrap_or_default();
        }

        if curl_options.verbose.unwrap_or_else(|| false) {
            debug!("Enabling Verbose mode.");
            curl.verbose(true).unwrap_or_default();
        } else {
            debug!("Disabling Verbose mode.");
            curl.verbose(false).unwrap_or_default();
        }

        // Setup Curl configuration based on given options
        match curl_options.method {
            Some(Method::PUT) | Some(Method::POST) => {
                debug!("Curl method: {}", "POST".magenta());
                let post_data
                    = curl_options
                        .post_data
                        .unwrap_or_default();
                curl
                    .get(false)
                    .unwrap_or_default();
                curl
                    .post(true)
                    .unwrap_or_default();
                curl
                    .post_field_size(post_data.len() as u64)
                    .unwrap_or_default();
            },

            // fallbacks to GET
            Some(_) | None => {
                debug!("Curl method: {}", "GET".magenta());
                curl
                    .put(false)
                    .unwrap_or_default();
                curl
                    .post(false)
                    .unwrap_or_default();
                curl
                    .get(true)
                    .unwrap_or_default();
            },
        };

        // Pass headers and cookies
        curl
            .http_headers(Self::list_of_headers(curl_options.headers))
            .unwrap_or_default();
        for cookie in
            curl_options
                .cookies
                .unwrap_or_default() {
            debug!("Setting cookie: {}", format!("{}", cookie.magenta()));
            curl
                .cookie(&cookie)
                .unwrap_or_default();
        }

        // Set agent
        match curl_options.agent {
            Some(new_agent) => {
                debug!("Setting useragent: {}", format!("{}", &new_agent.magenta()));
                curl
                    .useragent(&new_agent)
                    .unwrap_or_default()
            },
            None => {
                debug!("Empty useragent");
            }
        }

        // Set connection and request timeout with default fallback to 30s for each
        curl
            .connect_timeout(Duration::from_secs(curl_options.connection_timeout.unwrap_or_else(|| CHECK_CONNECTION_TIMEOUT)))
            .unwrap_or_default();
        curl
            .timeout(Duration::from_secs(curl_options.timeout.unwrap_or_else(|| CHECK_TIMEOUT)))
            .unwrap_or_default();

        // Verify SSL PEER
        if curl_options.ssl_verify_peer.unwrap_or_else(|| true) {
            debug!("Enabled TLS-PEER verification.");
            curl.ssl_verify_peer(true).unwrap_or_default();
        } else {
            warn!("Disabled TLS-PEER verification!");
            curl.ssl_verify_peer(false).unwrap_or_default();
        }

        // Verify SSL HOST
        if curl_options.ssl_verify_host.unwrap_or_else(|| true) {
            debug!("Enabled TLS-HOST verification.");
            curl.ssl_verify_host(true).unwrap_or_default();
        } else {
            warn!("Disabled TLS-HOST verification!");
            curl.ssl_verify_host(false).unwrap_or_default();
        }

        // Max connections is 10 per check
        curl.max_connects(CHECK_MAX_CONNECTIONS).unwrap_or_default();

        // Max reconnections is 10 per check
        curl.max_redirections(CHECK_MAX_REDIRECTIONS).unwrap_or_default();

        // let handler: CurlHandler = multi.add2(curl);
        multi.add2(curl)
    }


    /// Process Curl page requests using given handler
    fn process_page_handler(page_check: &Page, handler: CurlHandler, multi: &Multi) -> History {
        let page_expectations
            = page_check
                .clone()
                .expects
                .unwrap_or_else(Self::default_page_expectations);
        debug!("process_page_handler::page_expectations: {}",
               format!("{:?}", page_expectations).magenta());

        // take control over curl handler, perform validations, produce stories…
        let a_handler = match handler {
            Ok(handle) => handle,
            Err(err) => {
                error!("Curl-handler FAILURE: URL: {}. Error details: {:?}", page_check.url.cyan(), err.to_string().red());
                return History::empty()
            }
        };
        let handle = a_handler.get_ref();
        let raw_page_content = String::from_utf8_lossy(&handle.0);
        debug!("process_page_handler::raw_page_content: {}",
               format!("{}", raw_page_content).magenta());
        let expected_code = Self::find_code_validation(&page_expectations);
        debug!("process_page_handler::expected_code: {}",
               format!("{}", expected_code).magenta());
        let expected_content = Self::find_content_validation(&page_expectations);
        debug!("process_page_handler::expected_content: {}",
               format!("{}", expected_content).magenta());
        let expected_content_length = Self::find_content_length_validation(&page_expectations);
        debug!("process_page_handler::expected_content_length: {}",
               format!("{}", expected_content_length).magenta());
        let expected_final_address = Self::find_address_validation(&page_expectations);
        debug!("process_page_handler::expected_final_address: {}",
               format!("{}", expected_final_address).magenta());

        // Gather Story from expectations
        let content_story = Self::handle_page_content_expectation(&page_check.url, &raw_page_content, expected_content);
        debug!("process_page_handler::content_story: {}",
               format!("{:?}", content_story).magenta());
        let content_length_story = Self::handle_page_length_expectation(&page_check.url, &raw_page_content, expected_content_length);
        debug!("process_page_handler::content_length_story: {}",
               format!("{:?}", content_length_story).magenta());

        let mut result_handler = multi.remove2(a_handler).unwrap();
        let result_final_address = result_handler.effective_url().unwrap_or_default();
        let result_final_address_story = Self::handle_page_address_expectation(&page_check.url, &result_final_address.unwrap_or_default(), expected_final_address);
        debug!("process_page_handler::result_final_address_story: {}",
               format!("{:?}", result_final_address_story).magenta());
        let result_handler_story = Self::handle_page_httpcode_expectation(&page_check.url, result_handler.response_code().map_err(|err| Error::new(ErrorKind::Other, err.to_string())), expected_code);
        debug!("process_page_handler::handle_page_httpcode_expectation: {}",
               format!("{:?}", result_handler_story).magenta());

        // Collect the history results
        History::new_from(
            [
                History::new(content_story).stories(),
                History::new(content_length_story).stories(),
                History::new(result_handler_story).stories(),
                History::new(result_final_address_story).stories(),
            ].concat()
        )
    }


    /// Build a Story from a Length PageExpectation
    fn handle_page_content_expectation(url: &str, raw_page_content: &str, expected_content: &PageExpectation) -> Story {
        match expected_content {
            &PageExpectation::ValidContent(ref content) =>
                if raw_page_content.contains(content) {
                    Story::new(Some(Expected::Content(url.to_string(), content.to_string())))
                } else {
                    Story::new_error(Some(Unexpected::ContentInvalid(url.to_string(), content.to_string())))
                },

            &PageExpectation::ValidNoContent =>
                Story::new(Some(Expected::EmptyContent(url.to_string()))),

            edge_case =>
                Story::new_error(Some(Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string())))
        }
    }


    /// Build a Story from a Length PageExpectation
    fn handle_page_length_expectation(url: &str, raw_page_content: &str, expected_content_length: &PageExpectation) -> Story {
        match expected_content_length {
            &PageExpectation::ValidLength(ref requested_length) =>
                if raw_page_content.len() >= *requested_length {
                    Story::new(Some(Expected::ContentLength(url.to_string(), *requested_length)))
                } else {
                    Story::new_error(Some(Unexpected::ContentLengthInvalid(url.to_string(), raw_page_content.len(), *requested_length)))
                },

            &PageExpectation::ValidNoLength =>
                Story::new(Some(Expected::NoContentLength(url.to_string()))),

            edge_case =>
                Story::new_error(Some(Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string())))
        }
    }


    /// Build a Story from a Address PageExpectation
    fn handle_page_address_expectation(url: &str, address: &str, expected_address: &PageExpectation) -> Story {
        match expected_address {
            &PageExpectation::ValidAddress(ref an_address) =>
                if address.contains(an_address) {
                    Story::new(Some(Expected::Address(url.to_string(), address.to_string())))
                } else {
                    Story::new_error(Some(Unexpected::AddressInvalid(url.to_string(), address.to_string(), an_address.to_string())))
                },

            &PageExpectation::ValidNoAddress =>
                Story::new(Some(Expected::Address(url.to_string(), url.to_string()))),

            edge_case =>
                Story::new_error(Some(Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string())))
        }
    }


    /// Build a Story from a HttpCode PageExpectation
    fn handle_page_httpcode_expectation(url: &str, response_code: Result<u32, Error>, expected_code: &PageExpectation) -> Story {
        match response_code {
            Ok(responded_code) =>
                match expected_code {
                    &PageExpectation::ValidCode(the_code) if responded_code == the_code =>
                       Story::new(Some(Expected::HttpCode(url.to_string(), the_code))),

                    &PageExpectation::ValidCode(the_code) =>
                        Story::new_error(Some(Unexpected::HttpCodeInvalid(url.to_string(), responded_code, the_code))),

                    edge_case =>
                        Story::new_error(Some(Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string())))
                },

            Err(err) => Story::new_error(Some(Unexpected::URLConnectionProblem(url.to_string(), err.to_string())))
       }
    }


    /// Check pages
    fn check_pages(pages: Option<Pages>) -> History {
        let mut multi = Multi::new();
        multi.pipelining(true, true).unwrap_or_default();
        pages
            .and_then(|pages| {
                // collect tuple of page-checks and Curl handler:
                let process_handlers: Vec<_> // : Vec<(Page, CurlHandler)>
                    = pages
                        .iter()
                        .map(|check| (check, Self::load_handler_for(&check, &multi)))
                        .collect();

                // perform all checks at once:
                while multi
                        .perform()
                        .unwrap_or_default() > 0 {
                    multi
                        .wait(&mut [], Duration::from_secs(1))
                        .unwrap_or_default();
                }

                // Collect History of results:
                Some(
                     History::new_from(
                        process_handlers
                            .into_iter()
                            .flat_map(|(check, handler)| {
                                Self::process_page_handler(&check, handler, &multi)
                                    .stories()
                                    .into_par_iter()
                                    .map(|new_story| {
                                        match new_story.clone() {
                                            Story{ success: Some(success_msg), error: None, .. } =>
                                                info!("Web-page Story of: SUCCESS: {}", success_msg.to_string().green()),

                                            Story{ success: None, error: Some(error_msg), .. } =>
                                                error!("Web-page Story of: FAILURE: {}", error_msg.to_string().red()),

                                            Story{ success: None, error: None, .. } =>
                                                warn!("Web-page Story of: WARNING: {}", "Ambiguous Story that lacks both success and error?!".yellow()),

                                            Story{ success: Some(_), error: Some(_), .. } =>
                                                warn!("Web-page Story of: WARNING: {}", "Ambiguous Story with success and failure at the same time?!".yellow()),
                                        };
                                        new_story
                                    })
                                    .collect::<Vec<Story>>()
                            })
                            .collect()
                    )
                )
            })
            .unwrap_or_else(History::empty)
    }


}
