use curl::{
    easy::{Easy2, List},
    multi::{Easy2Handle, Multi},
    Error as CurlError, MultiError,
};
use rayon::prelude::*;
use ssl_expiration2::SslExpiration;
use std::{
    io::{Error, ErrorKind},
    time::Duration,
};

use crate::*;


/// Type alias for long type name:
pub type CurlHandler = Result<Easy2Handle<Collector>, MultiError>;


/// Checks trait
pub trait Checks<T> {
    /// Load check from any source
    fn load(name: &str) -> Result<T, Error>;


    /// Execute loaded checks
    fn execute(&self, execution_name: &str) -> History;


    /// Check SSL certificate expiration using OpenSSL function
    fn check_ssl_expire(domain_name: &str, domain_expectation: DomainExpectation) -> Story {
        SslExpiration::from_domain_name_with_timeout(&domain_name, CHECK_TIMEOUT)
            .map(|ssl_validator| {
                match domain_expectation {
                    DomainExpectation::ValidExpiryPeriod(expected_days)
                        if ssl_validator.days() < expected_days
                            || ssl_validator.is_expired() =>
                    {
                        Story::error(Unexpected::TLSDomainExpired(
                            domain_name.to_string(),
                            ssl_validator.days(),
                        ))
                    }

                    DomainExpectation::ValidExpiryPeriod(expected_days) => {
                        Story::success(Expected::TLSCertificateFresh(
                            domain_name.to_string(),
                            ssl_validator.days(),
                            expected_days,
                        ))
                    }
                }
            })
            .unwrap_or_else(|err| {
                Story::minor(UnexpectedMinor::InternalProtocolProblem(
                    domain_name.to_string(),
                    err.to_string(),
                ))
            })
    }


    /// Check domains
    fn check_domains(domains: Option<Domains>) -> History {
        domains.map(|domains| History::new_from(
                        domains
                            .into_par_iter()
                            .flat_map(|defined_check| {
                                let domain_check = defined_check;
                                let domain_name = domain_check.name;
                                let domain_expectations = domain_check.expects;
                                debug!("check_domain::domain_expectations: {}", format!("{:?}", domain_expectations).magenta());

                                // Process Domain expectations using parallel iterator (Rayon):
                                History::new_from(
                                    domain_expectations
                                        .into_iter()
                                        .map(|domain_expectation| {
                                            let domain_story = Self::check_ssl_expire(&domain_name, domain_expectation);
                                            match domain_story.clone() {
                                                Story{ success: Some(success_msg), error: None, .. } =>
                                                    info!("Domain TLS-cert validity Story of: SUCCESS: {}", success_msg.to_string().green()),

                                                Story{ success: None, error: Some(error_msg), .. } =>
                                                    error!("Domain TLS-cert validity Story of: FAILURE: {}", error_msg.to_string().red()),

                                                Story{ success: None, error: None, minor: Some(error_msg), .. } =>
                                                    warn!("Domain TLS-cert validity Story of: MINOR FAILURE: {}", error_msg.to_string().red()),

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
                        ))
            .unwrap_or_else(History::empty)
    }


    /// Find and extract code validation from validations
    fn find_code_validation(page_expectations: &[PageExpectation]) -> &PageExpectation {
        page_expectations
            .par_iter()
            .find_any(|exp| {
                match exp {
                    PageExpectation::ValidCode(_) => true,
                    _ => false,
                }
            })
            .unwrap_or_else(|| &PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE))
    }


    /// Find and extract content validations
    fn find_content_validations(page_expectations: &[PageExpectation]) -> PageExpectations {
        page_expectations
            .par_iter()
            .filter(|exp| {
                match exp {
                    PageExpectation::ValidContent(_) => true,
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }


    /// Find and extract content length validation from validations
    fn find_content_length_validation(
        page_expectations: &[PageExpectation],
    ) -> &PageExpectation {
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
            list.append(&header.to_owned()).unwrap_or_default();
        }
        list
    }


    /// Load page check handler
    fn load_handler_for(page_check: &Page, multi: &Multi) -> CurlHandler {
        // Initialize Curl, set URL
        let mut curl = Easy2::new(Collector(Vec::new()));
        curl.url(&page_check.url).unwrap_or_default();
        curl.useragent(&format!(
            "{name}/{version} (+github.com/verknowsys/krecik)",
            name = DEFAULT_SLACK_NAME,
            version = env!("CARGO_PKG_VERSION")
        ))
        .unwrap_or_default();
        debug!("Curl URL:: {}", format!("{}", &page_check.url.magenta()));

        // Load Curl request options from check:
        let curl_options = page_check.clone().options.unwrap_or_default();
        debug!(
            "Curl options: {}",
            format!("{}", curl_options.to_string().magenta())
        );

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
            Some(Method::PUT) => {
                debug!("Curl method: {}", "PUT".magenta());
                let post_data = curl_options.post_data.unwrap_or_default();
                curl.get(false).unwrap_or_default();
                curl.post(false).unwrap_or_default();
                curl.put(true).unwrap_or_default();
                curl.post_field_size(post_data.len() as u64)
                    .unwrap_or_default();
            }
            Some(Method::POST) => {
                debug!("Curl method: {}", "POST".magenta());
                let post_data = curl_options.post_data.unwrap_or_default();
                curl.get(false).unwrap_or_default();
                curl.put(false).unwrap_or_default();
                curl.post(true).unwrap_or_default();
                curl.post_field_size(post_data.len() as u64)
                    .unwrap_or_default();
            }

            // fallbacks to GET
            Some(_) | None => {
                debug!("Curl method: {}", "GET".magenta());
                curl.put(false).unwrap_or_default();
                curl.post(false).unwrap_or_default();
                curl.get(true).unwrap_or_default();
            }
        };

        // Pass headers and cookies
        curl.http_headers(Self::list_of_headers(curl_options.headers))
            .unwrap_or_default();
        for cookie in curl_options.cookies.unwrap_or_default() {
            debug!("Setting cookie: {}", format!("{}", cookie.magenta()));
            curl.cookie(&cookie).unwrap_or_default();
        }

        // Set agent
        match curl_options.agent {
            Some(new_agent) => {
                debug!("Setting useragent: {}", format!("{}", &new_agent.magenta()));
                curl.useragent(&new_agent).unwrap_or_default()
            }
            None => {
                debug!("Empty useragent");
            }
        }

        // Set connection and request timeout with default fallback to 30s for each
        curl.connect_timeout(Duration::from_secs(
            curl_options
                .connection_timeout
                .unwrap_or_else(|| CHECK_CONNECTION_TIMEOUT),
        ))
        .unwrap_or_default();
        curl.timeout(Duration::from_secs(
            curl_options.timeout.unwrap_or_else(|| CHECK_TIMEOUT),
        ))
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
        curl.max_redirections(CHECK_MAX_REDIRECTIONS)
            .unwrap_or_default();

        // let handler: CurlHandler = multi.add2(curl);
        multi.add2(curl)
    }


    /// Process Curl page requests using given handler
    fn process_page_handler(
        page_check: &Page,
        handler: CurlHandler,
        multi: &Multi,
    ) -> History {
        let page_expectations = page_check.clone().expects;
        debug!(
            "process_page_handler::page_expectations: {}",
            format!("{:?}", page_expectations).magenta()
        );

        // take control over curl handler, perform validations, produce stories…
        let a_handler = match handler {
            Ok(handle) => {
                if handle.get_ref().0.is_empty() {
                    warn!("Got an empty output from Curl handle!");
                    handle
                } else {
                    handle
                }
            }
            Err(err) => {
                error!(
                    "Curl-handler FAILURE: URL: {}. Error details: {:?}",
                    page_check.url.cyan(),
                    err.to_string().red()
                );
                return History::new(Story::error(Unexpected::HandlerFailed(
                    err.description().to_string(),
                )));
            }
        };

        let handle = a_handler.get_ref().0.to_owned();
        let raw_page_content = String::from_utf8(handle).unwrap_or_default();
        debug!(
            "process_page_handler::raw_page_content: {:?}",
            raw_page_content.magenta()
        );
        let expected_code = Self::find_code_validation(&page_expectations);
        debug!(
            "process_page_handler::expected_code: {}",
            format!("{}", expected_code).magenta()
        );
        let expected_contents = Self::find_content_validations(&page_expectations);
        debug!(
            "process_page_handler::expected_contents: {}",
            format!("{:?}", expected_contents).magenta()
        );
        let expected_content_length = Self::find_content_length_validation(&page_expectations);
        debug!(
            "process_page_handler::expected_content_length: {}",
            format!("{}", expected_content_length).magenta()
        );
        let expected_final_address = Self::find_address_validation(&page_expectations);
        debug!(
            "process_page_handler::expected_final_address: {}",
            format!("{}", expected_final_address).magenta()
        );

        // Gather Story from expectations
        let content_stories = Self::handle_page_content_expectations(
            &page_check.url,
            &raw_page_content,
            &expected_contents,
        );
        debug!(
            "process_page_handler::content_story: {}",
            format!("{:?}", content_stories).magenta()
        );
        let content_length_story = Self::handle_page_length_expectation(
            &page_check.url,
            &raw_page_content,
            expected_content_length,
        );
        debug!(
            "process_page_handler::content_length_story: {}",
            format!("{:?}", content_length_story).magenta()
        );

        let mut result_handler = match multi.remove2(a_handler) {
            Ok(res_handler) => res_handler,
            Err(err) => {
                error!(
                    "Curl-result-handler FAILURE: URL: {}. Error details: {:?}",
                    page_check.url.cyan(),
                    err.to_string().red()
                );
                return History::new(Story::error(Unexpected::HandlerFailed(
                    err.description().to_string(),
                )));
            }
        };
        let result_final_address = result_handler.effective_url().unwrap_or_default();
        let result_final_address_story = Self::handle_page_address_expectation(
            &page_check.url,
            &result_final_address.unwrap_or_default(),
            expected_final_address,
        );
        debug!(
            "process_page_handler::result_final_address_story: {}",
            format!("{:?}", result_final_address_story).magenta()
        );
        let connect_oserror = match result_handler.os_errno() {
            Ok(0) => None,
            Ok(err_code) => Some(Error::from_raw_os_error(err_code)),
            Err(error) => Some(Error::from_raw_os_error(error.code() as i32)),
        };
        debug!("Connect OS error: {:?}", connect_oserror);

        let result_handler_story = Self::handle_page_httpcode_expectation(
            &page_check.url,
            connect_oserror,
            result_handler
                .response_code()
                .map_err(Self::produce_curl_response_error),
            expected_code,
        );
        debug!(
            "process_page_handler::handle_page_httpcode_expectation: {}",
            format!("{:?}", result_handler_story).magenta()
        );

        // Collect the history results
        History::new_from(
            [
                content_stories,
                History::new(content_length_story).stories(),
                History::new(result_handler_story).stories(),
                History::new(result_final_address_story).stories(),
            ]
            .concat(),
        )
    }


    /// Converts CurlError to Error
    fn produce_curl_response_error(err: CurlError) -> Error {
        let mut reason = "";
        if err.is_failed_init() {
            reason = "CURLE_FAILED_INIT"
        } // Returns whether this error corresponds to CURLE_FAILED_INIT.
        if err.is_unsupported_protocol() {
            reason = "CURLE_UNSUPPORTED_PROTOCOL"
        } // Returns whether this error corresponds to CURLE_UNSUPPORTED_PROTOCOL.
        if err.is_couldnt_resolve_proxy() {
            reason = "CURLE_COULDNT_RESOLVE_PROXY"
        } // Returns whether this error corresponds to CURLE_COULDNT_RESOLVE_PROXY.
        if err.is_couldnt_resolve_host() {
            reason = "CURLE_COULDNT_RESOLVE_HOST"
        } // Returns whether this error corresponds to CURLE_COULDNT_RESOLVE_HOST.
        if err.is_couldnt_connect() {
            reason = "CURLE_COULDNT_CONNECT"
        } // Returns whether this error corresponds to CURLE_COULDNT_CONNECT.
        if err.is_remote_access_denied() {
            reason = "CURLE_REMOTE_ACCESS_DENIED"
        } // whether this error corresponds to CURLE_REMOTE_ACCESS_DENIED.
        if err.is_partial_file() {
            reason = "CURLE_PARTIAL_FILE"
        } // Returns whether this error corresponds to CURLE_PARTIAL_FILE.
        if err.is_quote_error() {
            reason = "CURLE_QUOTE_ERROR"
        } // Returns whether this error corresponds to CURLE_QUOTE_ERROR.
        if err.is_http_returned_error() {
            reason = "CURLE_HTTP_RETURNED_ERROR"
        } // Returns whether this error corresponds to CURLE_HTTP_RETURNED_ERROR.
        if err.is_read_error() {
            reason = "CURLE_READ_ERROR"
        } // Returns whether this error corresponds to CURLE_READ_ERROR.
        if err.is_write_error() {
            reason = "CURLE_WRITE_ERROR"
        } // Returns whether this error corresponds to CURLE_WRITE_ERROR.
        if err.is_out_of_memory() {
            reason = "CURLE_OUT_OF_MEMORY"
        } // Returns whether this error corresponds to CURLE_OUT_OF_MEMORY.
        if err.is_operation_timedout() {
            reason = "CURLE_OPERATION_TIMEDOUT"
        } // Returns whether this error corresponds to CURLE_OPERATION_TIMEDOUT.
        if err.is_ssl_connect_error() {
            reason = "CURLE_SSL_CONNECT_ERROR"
        } // Returns whether this error corresponds to CURLE_SSL_CONNECT_ERROR.
        if err.is_ssl_certproblem() {
            reason = "CURLE_SSL_CERTPROBLEM"
        } // Returns whether this error corresponds to CURLE_SSL_CERTPROBLEM.
        if err.is_ssl_cipher() {
            reason = "CURLE_SSL_CIPHER"
        } // Returns whether this error corresponds to CURLE_SSL_CIPHER.
        if err.is_ssl_cacert() {
            reason = "CURLE_SSL_CACERT"
        } // Returns whether this error corresponds to CURLE_SSL_CACERT.
        if err.is_ssl_engine_initfailed() {
            reason = "CURLE_SSL_ENGINE_INITFAILED"
        } // Returns whether this error corresponds to CURLE_SSL_ENGINE_INITFAILED.
        if err.is_ssl_issuer_error() {
            reason = "CURLE_SSL_ISSUER_ERROR"
        } // Returns whether this error corresponds to CURLE_SSL_ISSUER_ERROR.
        if err.is_too_many_redirects() {
            reason = "CURLE_TOO_MANY_REDIRECTS"
        } // Returns whether this error corresponds to CURLE_TOO_MANY_REDIRECTS.
        if err.is_peer_failed_verification() {
            reason = "CURLE_PEER_FAILED_VERIFICATION"
        } // Returns whether this error corresponds to CURLE_PEER_FAILED_VERIFICATION.
        if err.is_got_nothing() {
            reason = "CURLE_GOT_NOTHING"
        } // Returns whether this error corresponds to CURLE_GOT_NOTHING.
        if err.is_ssl_engine_notfound() {
            reason = "CURLE_SSL_ENGINE_NOTFOUND"
        } // Returns whether this error corresponds to CURLE_SSL_ENGINE_NOTFOUND.
        if err.is_ssl_engine_setfailed() {
            reason = "CURLE_SSL_ENGINE_SETFAILED"
        } // Returns whether this error corresponds to CURLE_SSL_ENGINE_SETFAILED.
        if err.is_send_error() {
            reason = "CURLE_SEND_ERROR"
        } // Returns whether this error corresponds to CURLE_SEND_ERROR.
        if err.is_recv_error() {
            reason = "CURLE_RECV_ERROR"
        } // Returns whether this error corresponds to CURLE_RECV_ERROR.
        if err.is_http2_stream_error() {
            reason = "CURLE_HTTP2_STREAM"
        } // Returns whether this error corresponds to CURLE_HTTP2_STREAM.
        if err.is_http2_error() {
            reason = "CURLE_HTTP2"
        } // Returns whether this error corresponds to CURLE_HTTP2.

        Error::new(ErrorKind::Other, format!("{} ({})", err, reason))
    }


    /// Build a Story from a Length PageExpectation
    fn handle_page_content_expectations(
        url: &str,
        raw_page_content: &str,
        expected_contents: &[PageExpectation],
    ) -> Stories {
        expected_contents
            .par_iter()
            .map(|expectation| {
                match expectation {
                    PageExpectation::ValidContent(ref content)
                        if raw_page_content.contains(content) =>
                    {
                        Story::success(Expected::Content(url.to_string(), content.to_string()))
                    }

                    PageExpectation::ValidContent(ref content) => {
                        Story::error(Unexpected::ContentInvalid(
                            url.to_string(),
                            content.to_string(),
                        ))
                    }

                    PageExpectation::ValidNoContent => {
                        Story::success(Expected::EmptyContent(url.to_string()))
                    }

                    edge_case => {
                        Story::error(Unexpected::UnmatchedValidationCase(
                            url.to_string(),
                            format!("{:?}", edge_case),
                        ))
                    }
                }
            })
            .collect::<Stories>()
    }


    /// Build a Story from a Length PageExpectation
    fn handle_page_length_expectation(
        url: &str,
        raw_page_content: &str,
        expected_content_length: &PageExpectation,
    ) -> Story {
        match expected_content_length {
            &PageExpectation::ValidLength(ref requested_length)
                if raw_page_content.len() >= *requested_length =>
            {
                Story::success(Expected::ContentLength(url.to_string(), *requested_length))
            }

            &PageExpectation::ValidLength(ref requested_length) => {
                Story::error(Unexpected::ContentLengthInvalid(
                    url.to_string(),
                    raw_page_content.len(),
                    *requested_length,
                ))
            }

            &PageExpectation::ValidNoLength => {
                Story::success(Expected::NoContentLength(url.to_string()))
            }

            edge_case => {
                Story::error(Unexpected::UnmatchedValidationCase(
                    url.to_string(),
                    edge_case.to_string(),
                ))
            }
        }
    }


    /// Build a Story from a Address PageExpectation
    fn handle_page_address_expectation(
        url: &str,
        address: &str,
        expected_address: &PageExpectation,
    ) -> Story {
        match expected_address {
            &PageExpectation::ValidAddress(ref an_address) if address.contains(an_address) => {
                Story::success(Expected::Address(url.to_string(), address.to_string()))
            }

            &PageExpectation::ValidAddress(ref an_address) => {
                Story::error(Unexpected::AddressInvalid(
                    url.to_string(),
                    address.to_string(),
                    an_address.to_string(),
                ))
            }

            &PageExpectation::ValidNoAddress => {
                Story::success(Expected::Address(url.to_string(), url.to_string()))
            }

            edge_case => {
                Story::error(Unexpected::UnmatchedValidationCase(
                    url.to_string(),
                    edge_case.to_string(),
                ))
            }
        }
    }


    /// Build a Story from a HttpCode PageExpectation
    fn handle_page_httpcode_expectation(
        url: &str,
        connect_oserror: Option<Error>,
        response_code: Result<u32, Error>,
        expected_code: &PageExpectation,
    ) -> Story {
        match response_code {
            Ok(responded_code) => {
                match expected_code {
                    &PageExpectation::ValidCode(the_code) if responded_code == the_code => {
                        Story::success(Expected::HttpCode(url.to_string(), the_code))
                    }

                    &PageExpectation::ValidCode(the_code)
                        if responded_code > 0 && responded_code != the_code =>
                    {
                        Story::error(Unexpected::HttpCodeInvalid(
                            url.to_string(),
                            responded_code,
                            the_code,
                        ))
                    }

                    &PageExpectation::ValidCode(_the_code) if responded_code == 0 => {
                        match connect_oserror {
                            Some(error) => {
                                Story::minor(UnexpectedMinor::OSError(
                                    url.to_string(),
                                    error.to_string(),
                                ))
                            }
                            None => {
                                Story::error(Unexpected::HttpConnectionFailed(
                                    url.to_string(),
                                    CHECK_CONNECTION_TIMEOUT,
                                ))
                            }
                        }
                    }

                    edge_case => {
                        Story::error(Unexpected::UnmatchedValidationCase(
                            url.to_string(),
                            edge_case.to_string(),
                        ))
                    }
                }
            }

            Err(err) => {
                Story::error(Unexpected::URLConnectionProblem(
                    url.to_string(),
                    err.to_string(),
                ))
            }
        }
    }


    /// Check pages
    fn check_pages(pages: Option<Pages>) -> History {
        let mut multi = Multi::new();
        // http1.1, http2-multiplex
        multi.pipelining(false, true).unwrap_or_default();
        pages
            .map(|pages| {
                // collect tuple of page-checks and Curl handler:
                // : Vec<(Page, CurlHandler)>
                let process_handlers: Vec<_> = pages
                    .iter()
                    .map(|check| (check, Self::load_handler_for(&check, &multi)))
                    .collect();

                // perform all checks at once:
                while multi.perform().unwrap_or_default() > 0 {
                    multi
                        .wait(&mut [], Duration::from_secs(CHECK_TIMEOUT))
                        .unwrap_or_default();
                }

                // Collect History of results:
                History::new_from(
                        process_handlers
                            .into_iter()
                            .flat_map(|(check, handler)|
                                Self::process_page_handler(&check, handler, &multi)
                                    .stories()
                                    .into_iter()
                                    .map(|new_story| {
                                        match new_story.clone() {
                                            Story{ success: Some(success_msg), error: None, .. } =>
                                                info!("Web-page Story of: SUCCESS: {}", success_msg.to_string().green()),

                                            Story{ success: None, error: Some(error_msg), .. } =>
                                                error!("Web-page Story of: FAILURE: {}", error_msg.to_string().red()),

                                            Story{ success: None, error: None, minor: Some(error_msg), .. } =>
                                                error!("Web-page Story of: MINOR FAILURE: {}", error_msg.to_string().red()),

                                            Story{ success: None, error: None, .. } =>
                                                warn!("Web-page Story of: WARNING: {}", "Ambiguous Story that lacks both success and error?!".yellow()),

                                            Story{ success: Some(_), error: Some(_), .. } =>
                                                warn!("Web-page Story of: WARNING: {}", "Ambiguous Story with success and failure at the same time?!".yellow()),
                                        };
                                        new_story
                                    })
                                    .collect::<Stories>()
                            )
                            .collect()
                    )
            })
            .unwrap_or_else(History::empty)
    }
}
