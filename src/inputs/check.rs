use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, List, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;
use colored::Colorize;


use crate::configuration::*;
use crate::utilities::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::products::history::*;


/// Collects async content from Curl:
struct Collector(Vec<u8>);


impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}


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
                    DomainExpectation::ValidExpiryPeriod(0) => {
                        let warn_msg = format!("Given ValidExpiryPeriod(0) for domain: {}. Validation skipped.", domain_name.cyan());
                        warn!("{}", warn_msg.yellow());
                        Err(warn_msg.into())
                    },

                    DomainExpectation::ValidExpiryPeriod(days) => {
                         if ssl_validator.days() < days
                         || ssl_validator.is_expired() {
                            let err_msg = Unexpected::TLSDomainExpired(domain_name.to_string()).to_string();
                            error!("{}", err_msg.red());
                            Err(err_msg.into())
                         } else {
                            let info_msg = Expected::TLSCertificateFresh(domain_name.to_string(), ssl_validator.days(), days);
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
                        .iter()
                        .flat_map(|defined_check| {
                            let domain_check = defined_check.clone();
                            let domain_name = domain_check.name;
                            let domain_expectations = domain_check
                                .expects
                                .unwrap_or_default();

                            History::new_from(domain_expectations
                                .iter()
                                .map(|domain_expectation| {
                                    Self::check_ssl_expire(&domain_name, *domain_expectation)
                                })
                                .collect()
                            ).stories()
                        })
                        .collect()
                    )
            },

            None => {
                debug!("Execute: No domains to check.");
                History::empty()
            }
        }
    }


    /// Check page expectations
    fn check_page(page_check: &Page) -> History {
        let mut history = History::empty();
        page_check
            .clone()
            .expects
            .and_then(|page_expectations| {
                let mut multi = Multi::new();
                multi.pipelining(true, true).unwrap();
                let handlers: Vec<_> = page_expectations
                    .iter()
                    .map(|_| {
                        // Initialize Curl, set URL
                        let mut curl = Easy2::new(Collector(Vec::new()));
                        curl.url(&page_check.url).unwrap();
                        debug!("Curl URL: {}", &page_check.url.cyan());

                        // Load Curl request options from check:
                        let curl_options = page_check.clone().options.unwrap_or_default();
                        debug!("Curl options: {}", curl_options.to_string().cyan());

                        // Setup Curl configuration based on given options
                        if curl_options.follow_redirects.unwrap_or_else(|| true) {
                            debug!("Enabled following redirects");
                            curl.follow_location(true).unwrap();
                        } else {
                            debug!("Disabled following redirects");
                            curl.follow_location(false).unwrap();
                        }

                        if curl_options.verbose.unwrap_or_else(|| false) {
                            debug!("Enabling Verbose mode");
                            curl.verbose(true).unwrap();
                        } else {
                            debug!("Disabling Verbose mode");
                            curl.verbose(false).unwrap();
                        }

                        // Setup Curl configuration based on given options
                        match curl_options.method {
                            Some(Method::PUT) | Some(Method::POST) => {
                                debug!("Curl method: PUT / POST");
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
                                debug!("Curl method: GET");
                                curl.get(true).unwrap();
                            },
                        };

                        //
                        // Build List of HTTP headers
                        //
                        // ex. header:
                        //         list.append("Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==").unwrap();
                        let mut list = List::new();
                        for header in curl_options
                                        .headers
                                        .unwrap_or_default() {
                            list
                                .append(&header.to_owned())
                                .unwrap();
                        }
                        curl
                            .http_headers(list)
                            .unwrap();

                        // Set connection and request timeout with default fallback to 30s for each
                        curl.connect_timeout(Duration::from_secs(curl_options.connection_timeout.unwrap_or_else(|| CHECK_CONNECTION_TIMEOUT))).unwrap();
                        curl.timeout(Duration::from_secs(curl_options.timeout.unwrap_or_else(|| CHECK_TIMEOUT))).unwrap();

                        // Verify SSL PEER
                        if curl_options.ssl_verify_peer.unwrap_or_else(|| true) {
                            debug!("Enabled TLS-PEER verification.");
                            curl.ssl_verify_peer(true).unwrap();
                        } else {
                            warn!("Disabled TLS-PEER verification!");
                            curl.ssl_verify_peer(false).unwrap();
                        }

                        // Verify SSL HOST
                        if curl_options.ssl_verify_host.unwrap_or_else(|| true) {
                            debug!("Enabled TLS-HOST verification.");
                            curl.ssl_verify_host(true).unwrap();
                        } else {
                            warn!("Disabled TLS-HOST verification!");
                            curl.ssl_verify_host(false).unwrap();
                        }

                        // Max connections is 10 per check
                        curl.max_connects(CHECK_MAX_CONNECTIONS).unwrap();

                        // Max reconnections is 10 per check
                        curl.max_redirections(CHECK_MAX_REDIRECTIONS).unwrap();

                        multi.add2(curl)
                    })
                    .collect();

                // perform async multicheck
                while multi.perform().unwrap() > 0 {
                    multi.wait(&mut [], Duration::from_secs(1)).unwrap();
                }

                // gather handlers after multicheck is finished, extract results
                for handler in handlers {
                    let a_handler = handler.unwrap();
                    let handle = a_handler.get_ref();
                    let expectations = page_check
                        .clone()
                        .expects
                        .unwrap_or_default();

                    // Error code expectation
                    let expected_code = expectations
                        .iter()
                        .find(|exp| {
                            let the_code = match exp {
                                PageExpectation::ValidCode(code) => code,
                                _ => &CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE,
                            };
                            the_code != &CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE
                        })
                        .unwrap_or_else(|| &PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE));

                    // Content expectation
                    let empty_content = PageExpectation::ValidContent("".to_string());
                    let expected_content = expectations
                        .iter()
                        .find(|exp| {
                            let the_content = match exp {
                                PageExpectation::ValidContent(content) => content,
                                _ => "",
                            };
                            the_content != ""
                        })
                        .unwrap_or_else(|| &empty_content);

                    let raw_page_content = String::from_utf8_lossy(&handle.0);
                    match expected_content {
                        &PageExpectation::ValidContent(ref content) if !content.is_empty() => {
                            if raw_page_content.contains(content) {
                                let info_msg = Expected::ContentValid(page_check.url.to_string(), content.to_string());
                                info!("{}", info_msg.to_string().green());
                                history = history.append(Story::new(Some(info_msg)))
                            }
                        },

                        &PageExpectation::ValidContent(ref content) if content.is_empty() => {
                            let dbg_msg = format!("Validation of an empty content from URL: {}", page_check.url.cyan());
                            debug!("{}", dbg_msg);
                        },

                        edge_case => {
                            let warn_msg = Unexpected::NotImplementedYet(page_check.url.to_string(), edge_case.to_string());
                            warn!("{}", warn_msg.to_string().yellow());
                            history = history.append(Story::new_error(Some(warn_msg)))
                        }
                    }

                    // Content length validation
                    let expected_content_length = expectations
                        .iter()
                        .find(|exp| {
                            let the_content = match exp {
                                PageExpectation::ValidLength(length) => length,
                                _ => &0usize,
                            };
                            the_content != &0usize
                        })
                        .unwrap_or_else(|| &PageExpectation::ValidLength(0usize));

                    match expected_content_length {
                        &PageExpectation::ValidLength(0) => {
                            let dbg_msg = format!("Got zero-length content for URL: {}. ValidLength(0) will be ignored.", &page_check.url.cyan());
                            debug!("{}", dbg_msg);
                        },

                        &PageExpectation::ValidLength(ref requested_length) => {
                            if raw_page_content.len() >= *requested_length {
                                let info_msg = Expected::ContentLength(page_check.url.to_string(), *requested_length);
                                info!("{}", info_msg.to_string().green());
                                history = history.append(Story::new(Some(info_msg)))
                            } else {
                                let unexpected = Unexpected::MinimumContentLength(page_check.url.to_string(), *requested_length, raw_page_content.len());
                                error!("{}", unexpected.to_string().red());
                                history = history.append(Story::new_error(Some(unexpected)));
                            }
                        },

                        edge_case => {
                            let warn_msg = Unexpected::NotImplementedYet(page_check.url.to_string(), edge_case.to_string());
                            warn!("{}", warn_msg.to_string().yellow());
                            history = history.append(Story::new_error(Some(warn_msg)));
                        },
                    }

                    let mut result_handler = multi.remove2(a_handler).unwrap();
                    match result_handler.response_code() {
                        Ok(0) => {
                            let unexpected = Unexpected::HttpErrorCode(page_check.url.to_string(), 0, 0);
                            error!("{}", unexpected.to_string().red());
                            history = history.append(Story::new_error(Some(unexpected)));
                        },

                        Ok(code) => {
                            if &PageExpectation::ValidCode(code) == expected_code {
                                let info_msg = Expected::HttpCodeValid(page_check.url.to_string(), code);
                                info!("{}", info_msg.to_string().green());
                                history = history.append(Story::new(Some(info_msg)));

                            } else if let PageExpectation::ValidCode(ref expectation_code) = expected_code {
                                let unexpected = Unexpected::HttpErrorCode(page_check.url.to_string(), code, *expectation_code);
                                error!("{}", unexpected.to_string().red());
                                history = history.append(Story::new_error(Some(unexpected)));
                            }
                        },

                        Err(err) => {
                            let unexpected = Unexpected::URLConnectionProblem(page_check.url.to_string(), err.to_string());
                            error!("{}", unexpected.to_string().red());
                            history = history.append(Story::new_error(Some(unexpected)));
                        }
                    }
                }
                Some(history)
            })
            .unwrap_or_else(|| {
                debug!("History is empty.");
                History::empty()
            })
    }


    /// Check pages
    fn check_pages(pages: Option<Pages>) -> History {
        match pages {
            Some(pages) => {
                History::new_from(
                    pages
                        .iter()
                        .flat_map(|check| {
                            Self::check_page(&check).stories()
                        })
                        .collect()
                    )
            },

            None => {
                debug!("Execute: No pages to check.");
                History::empty()
            }
        }
    }


}
