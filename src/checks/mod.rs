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
             .and_then(|ssl_validator| {
                 match domain_expectation {
                    DomainExpectation::ValidExpiryPeriod(expected_days) => {
                         if ssl_validator.days() < expected_days
                         || ssl_validator.is_expired() {
                            let err_msg = Unexpected::TLSDomainExpired(domain_name.to_string()).to_string();
                            error!("{}", err_msg.red());
                            Err(err_msg.into())
                         } else {
                            let info_msg = Expected::TLSCertificateFresh(domain_name.to_string(), ssl_validator.days(), expected_days);
                            info!("{}", info_msg.to_string().green());
                            Ok(Story::new(Some(info_msg)))
                         }
                     }
                 }
             })
             .unwrap_or_else(|err| {
                let unexpected = Unexpected::InternalProtocolProblem(domain_name.to_string(), err.0.to_string());
                error!("{}", unexpected.to_string().red());
                Story::new_error(Some(unexpected))
             })
    }


    /// Check domains
    fn check_domains(domains: Option<Domains>) -> History {
        match domains {
            Some(domains) => {
                History::new_from(
                    domains
                        .into_par_iter()
                        .flat_map(|defined_check| {
                            let domain_check = defined_check.clone();
                            let domain_name = domain_check.name;
                            let domain_expectations = domain_check
                                .expects
                                .unwrap_or_else(Self::default_domain_expectations);
                            let debugmsg = format!("check_domain::domain_expectations -> {:#?}", domain_expectations);
                            debug!("{}", debugmsg.magenta());

                            // Process Domain expectations using parallel iterator (Rayon):
                            History::new_from(
                                domain_expectations
                                    .into_par_iter()
                                    .map(|domain_expectation| {
                                        Self::check_ssl_expire(&domain_name, domain_expectation)
                                    })
                                    .collect()
                            ).stories()
                        })
                        .collect()
                    )
            },

            None => {
                debug!("{}", "Execute: No domains to check.".black());
                History::empty()
            }
        }
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
            debug!("{}", format!("Setting header: {}", header.cyan()).black());
            list
                .append(&header.to_owned())
                .unwrap();
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
        curl.url(&page_check.url).unwrap();
        debug!("{}", format!("Curl URL: {}", &page_check.url.cyan()).black());

        // Load Curl request options from check:
        let curl_options = page_check.clone().options.unwrap_or_default();
        debug!("{}", format!("Curl options: {}", curl_options.to_string().cyan()).black());

        // Setup Curl configuration based on given options
        if curl_options.follow_redirects.unwrap_or_else(|| true) {
            debug!("{}", "Enabled following redirects".black());
            curl.follow_location(true).unwrap();
        } else {
            debug!("{}", "Disabled following redirects".black());
            curl.follow_location(false).unwrap();
        }

        if curl_options.verbose.unwrap_or_else(|| false) {
            debug!("{}", "Enabling Verbose mode".black());
            curl.verbose(true).unwrap();
        } else {
            debug!("{}", "Disabling Verbose mode".black());
            curl.verbose(false).unwrap();
        }

        // Setup Curl configuration based on given options
        match curl_options.method {
            Some(Method::PUT) | Some(Method::POST) => {
                debug!("{}", "Curl method: PUT / POST".black());
                let post_data = curl_options
                                    .post_data
                                    .unwrap_or_default();
                curl
                    .post(true)
                    .unwrap();
                curl
                    .post_field_size(post_data.len() as u64)
                    .unwrap();
            },

            // fallbacks to GET
            Some(_) | None => {
                debug!("{}", "Curl method: GET".black());
                curl.get(true).unwrap();
            },
        };

        curl
            .http_headers(Self::list_of_headers(curl_options.headers))
            .unwrap();

        // Pass cookies
        for cookie in curl_options
                        .cookies
                        .unwrap_or_default() {
            debug!("{}", format!("Setting cookie: {}", cookie.cyan()).black());
            curl
                .cookie(&cookie)
                .unwrap();
        }

        // Set agent
        match curl_options.agent {
            Some(new_agent) => {
                debug!("{}", format!("Setting useragent: {}", &new_agent.cyan()).black());
                curl
                    .useragent(&new_agent)
                    .unwrap()
            },
            None => {
                debug!("{}", "Empty useragent".black());
            }
        }

        // Set connection and request timeout with default fallback to 30s for each
        curl.connect_timeout(Duration::from_secs(curl_options.connection_timeout.unwrap_or_else(|| CHECK_CONNECTION_TIMEOUT))).unwrap();
        curl.timeout(Duration::from_secs(curl_options.timeout.unwrap_or_else(|| CHECK_TIMEOUT))).unwrap();

        // Verify SSL PEER
        if curl_options.ssl_verify_peer.unwrap_or_else(|| true) {
            debug!("{}", "Enabled TLS-PEER verification.".black());
            curl.ssl_verify_peer(true).unwrap();
        } else {
            warn!("Disabled TLS-PEER verification!");
            curl.ssl_verify_peer(false).unwrap();
        }

        // Verify SSL HOST
        if curl_options.ssl_verify_host.unwrap_or_else(|| true) {
            debug!("{}", "Enabled TLS-HOST verification.".black());
            curl.ssl_verify_host(true).unwrap();
        } else {
            warn!("Disabled TLS-HOST verification!");
            curl.ssl_verify_host(false).unwrap();
        }

        // Max connections is 10 per check
        curl.max_connects(CHECK_MAX_CONNECTIONS).unwrap();

        // Max reconnections is 10 per check
        curl.max_redirections(CHECK_MAX_REDIRECTIONS).unwrap();

        // let handler: CurlHandler = multi.add2(curl);
        multi.add2(curl)
    }


    /// Process Curl page requests using given handler
    fn process_page_handler(page_check: &Page, handler: CurlHandler, multi: &Multi) -> History {
        let page_expectations = page_check
            .clone()
            .expects
            .unwrap_or_else(Self::default_page_expectations);
        let debugmsg = format!("process_page_handler::page_expectations -> {:#?}", page_expectations);
        debug!("{}", debugmsg.magenta());

        // gather handlers, perform validations, produce stories…
        let a_handler = handler.unwrap();
        let handle = a_handler.get_ref();
        let raw_page_content = String::from_utf8_lossy(&handle.0);

        let expected_code = Self::find_code_validation(&page_expectations);
        let expected_content = Self::find_content_validation(&page_expectations);
        let expected_content_length = Self::find_content_length_validation(&page_expectations);
        let expected_final_address = Self::find_address_validation(&page_expectations);

        // Gather Story from expectations
        let content_story = Self::handle_page_content_expectation(&page_check.url, &raw_page_content, expected_content);
        let content_length_story = Self::handle_page_length_expectation(&page_check.url, &raw_page_content, expected_content_length);

        let mut result_handler = multi.remove2(a_handler).unwrap();
        let result_final_address = result_handler.effective_url().unwrap_or_default();
        let result_final_address_story = Self::handle_page_address_expectation(&page_check.url, &result_final_address.unwrap_or_default(), expected_final_address);
        let result_handler_story = Self::handle_page_httpcode_expectation(&page_check.url, result_handler.response_code().map_err(|err| Error::new(ErrorKind::Other, err.to_string())), expected_code);

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
            &PageExpectation::ValidContent(ref content) => {
                if raw_page_content.contains(content) {
                    let info_msg = Expected::Content(url.to_string(), content.to_string());
                    info!("{}", info_msg.to_string().green());
                    Story::new(Some(info_msg))
                } else {
                    let error_msg = Unexpected::ContentInvalid(url.to_string(), content.to_string());
                    error!("{}", error_msg.to_string().red());
                    Story::new_error(Some(error_msg))
                }
            },

            &PageExpectation::ValidNoContent => {
                let info_msg = Expected::EmptyContent(url.to_string());
                info!("{}", info_msg.to_string().green());
                Story::new(Some(info_msg))
            },

            edge_case => {
                let warn_msg = Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string());
                warn!("{}", warn_msg.to_string().yellow());
                Story::new_error(Some(warn_msg))
            }
        }
    }


    /// Build a Story from a Length PageExpectation
    fn handle_page_length_expectation(url: &str, raw_page_content: &str, expected_content_length: &PageExpectation) -> Story {
        match expected_content_length {
            &PageExpectation::ValidLength(ref requested_length) => {
                if raw_page_content.len() >= *requested_length {
                    let info_msg = Expected::ContentLength(url.to_string(), *requested_length);
                    info!("{}", info_msg.to_string().green());
                    Story::new(Some(info_msg))
                } else {
                    let unexpected = Unexpected::ContentLengthInvalid(url.to_string(), raw_page_content.len(), *requested_length);
                    error!("{}", unexpected.to_string().red());
                    Story::new_error(Some(unexpected))
                }
            },

            &PageExpectation::ValidNoLength => {
                let info_msg = Expected::NoContentLength(url.to_string());
                info!("{}", info_msg.to_string().green());
                Story::new(Some(info_msg))
            },

            edge_case => {
                let warn_msg = Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string());
                warn!("{}", warn_msg.to_string().yellow());
                Story::new_error(Some(warn_msg))
            },
        }
    }


    /// Build a Story from a Address PageExpectation
    fn handle_page_address_expectation(url: &str, address: &str, expected_address: &PageExpectation) -> Story {
        match expected_address {
            &PageExpectation::ValidAddress(ref an_address) => {
                if address.contains(an_address) {
                    let info_msg = Expected::Address(url.to_string(), address.to_string());
                    info!("{}", info_msg.to_string().green());
                    Story::new(Some(info_msg))
                } else {
                    let error_msg = Unexpected::AddressInvalid(url.to_string(), address.to_string(), an_address.to_string());
                    error!("{}", error_msg.to_string().red());
                    Story::new_error(Some(error_msg))
                }
            },

            &PageExpectation::ValidNoAddress => {
                let info_msg = Expected::Address(url.to_string(), url.to_string());
                info!("{}", info_msg.to_string().green());
                Story::new(Some(info_msg))
            },

            edge_case => {
                let warn_msg = Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string());
                warn!("{}", warn_msg.to_string().yellow());
                Story::new_error(Some(warn_msg))
            }
        }
    }


    /// Build a Story from a HttpCode PageExpectation
    fn handle_page_httpcode_expectation(url: &str, response_code: Result<u32, Error>, expected_code: &PageExpectation) -> Story {
        match response_code {
            Ok(responded_code) => {
                match expected_code {
                    &PageExpectation::ValidCode(the_code) if responded_code == the_code => {
                       let info_msg = Expected::HttpCode(url.to_string(), the_code);
                       info!("{}", info_msg.to_string().green());
                       Story::new(Some(info_msg))
                    },

                    &PageExpectation::ValidCode(the_code) => {
                        let unexpected = Unexpected::HttpCodeInvalid(url.to_string(), responded_code, the_code);
                        error!("{}", unexpected.to_string().red());
                        Story::new_error(Some(unexpected))
                    },

                    edge_case => {
                        let warn_msg = Unexpected::UnmatchedValidationCase(url.to_string(), edge_case.to_string());
                        warn!("{}", warn_msg.to_string().yellow());
                        Story::new_error(Some(warn_msg))
                    }
                }
            },

            Err(err) => {
                let unexpected = Unexpected::URLConnectionProblem(url.to_string(), err.to_string());
                error!("{}", unexpected.to_string().red());
                Story::new_error(Some(unexpected))
            }
       }
    }


    /// Check pages
    fn check_pages(pages: Option<Pages>) -> History {
        let mut multi = Multi::new();
        multi.pipelining(true, true).unwrap();
        match pages {
            Some(pages) => {
                // collect tuple of page-checks and Curl handler:
                let process_handlers: Vec<_> // : Vec<(Page, CurlHandler)>
                    = pages
                        .iter()
                        .map(|check| (check, Self::load_handler_for(&check, &multi)))
                        .collect();

                // perform all checks at once:
                while multi
                        .perform()
                        .unwrap() > 0 {
                    multi
                        .wait(&mut [], Duration::from_secs(1))
                        .unwrap();
                }

                // Collect History of results:
                History::new_from(
                    process_handlers
                        .into_iter()
                        .flat_map(|(check, handler)| {
                            Self::process_page_handler(&check, handler, &multi).stories()
                        })
                        .collect()
                )
            },

            None => {
                debug!("{}", "Execute: No pages to check.".black());
                History::empty()
            }
        }
    }


}
