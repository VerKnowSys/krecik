use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;

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
    fn execute(&self) -> Result<History, History>;


    /// Check SSL certificate expiration using OpenSSL function
    fn check_ssl_expire(domain_name: &str, domain_expectation: &DomainExpectation) -> Story {
         SslExpiration::from_domain_name(&domain_name)
             .and_then(|ssl_validator| {
                 match domain_expectation {
                     DomainExpectation::ValidExpiryPeriod(days) => {
                         debug!("Validating expectation: ValidExpiryPeriod({} days) for domain: {}", days, domain_name);
                         if &ssl_validator.days() < &days
                         || ssl_validator.is_expired() {
                             error!("Expired domain: {}.", domain_name);
                             Err(format!("Expired domain: {}.", domain_name).into())
                         } else {
                             debug!("Requested domain: {} to be valid for: {} days. Domain will remain valid for {} days.",
                                    domain_name, days, ssl_validator.days());
                             Ok(Story::new(Some(format!("SSL for domain: {} is valid for {} days", domain_name, ssl_validator.days()))))
                         }
                     },

                     _ => {
                         debug!("Validating expectation: ValidResolvable for domain: {}", domain_name);
                         if ssl_validator.is_expired() {
                             error!("Expired domain: {}.", domain_name);
                             Err(format!("Expired domain: {}.", domain_name).into())
                         } else {
                             Ok(Story::new(Some(format!("Domain: {} validation successful", domain_name))))
                         }
                     }
                 }
             })
             .unwrap_or_else(|_| {
                let error_msg = format!("Internal OpenSSL/ Protocol error for domain: {}!", domain_name);
                error!("{}", error_msg);
                Story::new_error(Some(Unexpected::FailedInternal(error_msg)))
             })
    }


    /// Check domains
    fn check_domains(domains: Option<Domains>) -> Result<History, History> {
        let mut history = History::empty();
        match domains {
            Some(domains) => {
                domains
                    .iter()
                    .for_each(|defined_check| {
                        let domain_check = defined_check.clone();
                        let domain_name = domain_check.name;
                        domain_check
                            .expects
                            .and_then(|domain_expectations| {
                                domain_expectations
                                    .iter()
                                    .for_each(|domain_expectation| {
                                        history = history.append(Self::check_ssl_expire(&domain_name, &domain_expectation));
                                    });
                                Some(())
                            });
                        }
                    );
            },

            None => {
                debug!("Execute: No domains to check.");
            }
        };

        Ok(history)
    }


    /// Check page expectations
    fn check_page(page_url: &str, page_check: &Page) -> History {
        let mut history = History::empty();
        page_check
            .clone()
            .expects
            .and_then(|page_expectations| {
                let mut multi = Multi::new();
                multi.pipelining(true, true).unwrap();
                let handlers: Vec<_> = page_expectations
                    .iter()
                    .map(|page_expectation| {
                        // Initialize Curl, set URL
                        let mut curl = Easy2::new(Collector(Vec::new()));
                        curl.url(&page_url).unwrap();

                        // Load Curl request options from check:
                        let curl_options = page_check.clone().options.unwrap_or_default();
                        debug!("Curl options: {:?}", curl_options);

                        // Setup Curl configuration based on given options
                        if curl_options.follow_redirects.unwrap_or_default() {
                            debug!("Following 30x");
                            curl.follow_location(true).unwrap();
                        } else {
                            debug!("NOT Following 30x");
                            curl.follow_location(false).unwrap();
                        }

                        // Setup Curl configuration based on given options
                        match curl_options.method {
                            Some(Method::PUT) => curl.put(true).unwrap(),
                            Some(Method::POST) => curl.post(true).unwrap(),

                            // fallbacks to GET
                            Some(_) => curl.get(true).unwrap(),
                            None => curl.get(true).unwrap(),
                        };

                        // Set connection and request timeout with default fallback to 30s for each
                        curl.connect_timeout(Duration::from_secs(curl_options.connection_timeout.unwrap_or_else(|| CHECK_CONNECTION_TIMEOUT))).unwrap();
                        curl.timeout(Duration::from_secs(curl_options.timeout.unwrap_or_else(|| CHECK_TIMEOUT))).unwrap();

                        // Verify SSL peer and host by default:
                        curl.ssl_verify_peer(true).unwrap();
                        curl.ssl_verify_host(true).unwrap();

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
                                _ => &0u32,
                            };
                            the_code != &0u32
                        })
                        .unwrap_or_else(|| &PageExpectation::ValidCode(0)); // code 0 means connection error - we may want to check if page just fails

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
                        &PageExpectation::ValidContent(ref content) => {
                            if content.len() > 0 {
                                if raw_page_content.contains(content) {
                                    let ok_msg = format!("Got expected content: '{}' from URL: '{}'", content, page_url);
                                    info!("{}", ok_msg);
                                    history = history.append(Story::new(Some(ok_msg)))
                                } else {
                                    let err_msg = format!("Failed to find content: '{}' from URL: '{}'", content, page_url);
                                    error!("{}", err_msg);
                                    history = history.append(Story::new_error(Some(Unexpected::FailedPage(err_msg))));
                                }
                            } else {
                                debug!("Skipping validation of match for empty content!");
                            }
                        },

                        _ => {
                            let other_msg = "Some other case".to_string();
                            debug!("{}", other_msg);
                            history = history.append(Story::new(Some(other_msg)))
                        },
                    }

                    let mut result_handler = multi.remove2(a_handler).unwrap();
                    match result_handler.response_code() {
                        Ok(0) => {
                            let err_msg = format!("Error connecting to URL: {}", page_url);
                            error!("{}", err_msg);
                            history = history.append(Story::new_error(Some(Unexpected::FailedPage(err_msg))));
                        },

                        Ok(code) => {
                            if &PageExpectation::ValidCode(code) == expected_code {
                                let info_msg = format!("Got expected code: {} from URL: {}", code, page_url);
                                info!("{}", info_msg);
                                history = history.append(Story::new(Some(info_msg)));
                            } else {
                                let err_msg = format!("Got Unexpected code: {} from URL: {}", code, page_url);
                                error!("{}", err_msg);
                                history = history.append(Story::new_error(Some(Unexpected::FailedPage(err_msg))));
                            }
                        },

                        Err(err) => {
                            let err_msg = format!("Got unexpected error: {} from URL: {}", err, page_url);
                            error!("{}", err_msg);
                            history = history.append(Story::new_error(Some(Unexpected::FailedPage(err_msg))));
                        }
                    }
                }
                Some(history)
            })
            .unwrap_or_else(|| {
                debug!("Faillback to empty history");
                History::empty()
            })
    }


    /// Check pages
    fn check_pages(pages: Option<Pages>) -> Result<History, History> {
        let mut history = History::empty();
        match pages {
            Some(pages) => {
                pages
                    .iter()
                    .for_each(|defined_page| {
                        let page_check = defined_page.clone();
                        let page_url = page_check.url.clone();
                        history = history.merge(Self::check_page(&page_url, &page_check));
                    });
            },

            None => {
                debug!("Execute: No domains to check.");
            }
        }

        Ok(history)
    }


}
