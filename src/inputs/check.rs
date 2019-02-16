use crate::checks::page::Pages;
use crate::checks::domain::Domains;
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
    fn execute(&self) -> Result<(), History>;


    /// Check domains
    fn check_domains(domains: Option<Domains>) -> Result<(), History> {
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
                                        SslExpiration::from_domain_name(&domain_name)
                                            .and_then(|ssl_validator| {
                                                match domain_expectation {
                                                    DomainExpectation::ValidExpiryPeriod(days) => {
                                                        debug!("Validating expectation: ValidExpiryPeriod({} days) for domain: {}", days, domain_name);
                                                        if &ssl_validator.days() < days
                                                        || ssl_validator.is_expired() {
                                                            error!("Expired domain: {}.", domain_name);
                                                        } else {
                                                            debug!("Requested domain: {} to be valid for: {} days. Domain will remain valid for {} days.",
                                                                   domain_name, days, ssl_validator.days());
                                                        }
                                                        Ok(())
                                                    },

                                                    _ => {
                                                        debug!("Validating expectation: ValidResolvable for domain: {}", domain_name);
                                                        if ssl_validator.is_expired() {
                                                            error!("Expired domain: {}.", domain_name);
                                                        }
                                                        Ok(())
                                                    }
                                                }
                                            })
                                            .unwrap_or_else(|err| {
                                                error!("Internal OpenSSL/ Protocol error for domain: {}! Details: {:?}", domain_name, err);
                                            });
                                    });

                                Some(())
                            })
                            .unwrap_or_default();
                        }
                    )
            },

            None => {
                debug!("Execute: No domains to check.");
            }
        }

        Ok(())
    }


    /// Check pages
    fn check_pages(pages: Option<Pages>) -> Result<(), History> {
        match pages {
            Some(pages) => {
                pages
                    .iter()
                    .for_each(|defined_page| {
                        let page_check = defined_page.clone();
                        let page_url = page_check.url.clone();
                        page_check
                            .clone()
                            .expects
                            .and_then(|page_expectations| {
                                let mut multi = Multi::new();
                                multi.pipelining(true, true).unwrap();
                                let handlers: Vec<_> = page_expectations
                                    .iter()
                                    .map(|page_expectation| {
                                        let mut curl = Easy2::new(Collector(Vec::new()));
                                        // todo: use options field to set wanted options, leaving default for now:
                                        curl.url(&page_url).unwrap();
                                        curl.get(true).unwrap();
                                        curl.follow_location(true).unwrap();
                                        curl.ssl_verify_peer(true).unwrap();
                                        curl.ssl_verify_host(true).unwrap();
                                        curl.connect_timeout(Duration::from_secs(30)).unwrap();
                                        curl.timeout(Duration::from_secs(30)).unwrap();
                                        curl.max_connects(10).unwrap();
                                        curl.max_redirections(10).unwrap();
                                        multi.add2(curl)
                                    })
                                    .collect();

                                // perform async multicheck
                                while multi.perform().unwrap() > 0 {
                                    multi.wait(&mut [], Duration::from_secs(1)).unwrap();
                                }

                                for handler in handlers {
                                    let a_handler = handler.unwrap();
                                    let handle = a_handler.get_ref();
                                    let expectations = page_check
                                        .clone()
                                        .expects
                                        .unwrap_or_default();

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
                                            if raw_page_content.contains(content) {
                                                info!("Got expected content: '{}' from URL: '{}'", content, page_url);
                                            } else {
                                                error!("Failed to find content: '{}' from URL: '{}'", content, page_url);
                                            }
                                        },

                                        _ => {
                                            debug!("Some other case");
                                        },
                                    }

                                    let mut result_handler = multi.remove2(a_handler).unwrap();
                                    match result_handler.response_code() {
                                        Ok(0) => {
                                            error!("Error connecting to URL: {}", page_url);
                                        },

                                        Ok(code) => {
                                            if &PageExpectation::ValidCode(code) == expected_code {
                                                info!("Got expected code: {} from URL: {}", code, page_url);
                                            } else {
                                                error!("Got UNexpected code: {} from URL: {}", code, page_url);
                                            }
                                        },

                                        Err(err) => {
                                            error!("Got unexpected error: {}", err);
                                        }
                                    }
                                }

                                Some(())
                            })
                            .unwrap_or_default();
                        }
                    )
            },

            None => {
                debug!("Execute: No domains to check.");
            }
        }

        Ok(())
    }


}
