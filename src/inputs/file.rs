use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;

use crate::configuration::*;
use crate::utilities::*;
use crate::inputs::check::*;
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


/// NOTE: Pigeon (previous implementation) supported list of checks per file. TravMole will require each JSON to be separate file.
///       Decission is justified by lack of JSON comment ability, and other file-specific and sync troubles,
///       but also for future editing/ enable/ disable abilities that would be much more complicated with support of several checks per file.


#[derive(Debug, Clone, Serialize, Deserialize)]
/// FileCheck structure
pub struct FileCheck {

    /// Unique check name
    pub name: Option<String>,

    /// Domains to check
    pub domains: Option<Domains>,

    /// Pages to check
    pub pages: Option<Pages>,

    /// Slack Webhook
    pub alert_webhook: Option<String>,

    /// Slack alert channel
    pub alert_channel: Option<String>,

}


impl Checks<FileCheck> for FileCheck {


    fn load(name: &str) -> Result<FileCheck, Error> {
        let check_file = format!("{}/{}.json", CHECKS_DIR, &name);
        read_text_file(&check_file)
            .and_then(|file_contents| {
                serde_json::from_str(&file_contents.to_string())
                    .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
            })
    }


    fn execute(&self) -> Result<(), History> {
        match &self.domains {
            Some(domains) => {
                domains
                    .iter()
                    .for_each(|defined_check| {
                        let domain_check = defined_check.clone();
                        let domain_name = domain_check.name.unwrap_or_default();
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
                                                        if days < &ssl_validator.days()
                                                        || ssl_validator.is_expired() {
                                                            error!("Expired domain: {}.", domain_name);
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
                                            .unwrap_or_else(|_| {
                                                error!("Internal/ Protocol error on validating domain: {}!", domain_name);
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

        match &self.pages {
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
                                        .unwrap();

                                    let expected_content = expectations
                                        .iter()
                                        .find(|exp| {
                                            let the_content = match exp {
                                                PageExpectation::ValidContent(content) => content,
                                                _ => "",
                                            };
                                            the_content != ""
                                        })
                                        .unwrap();

                                    let raw_page_content = String::from_utf8_lossy(&handle.0);
                                    match expected_content {
                                        &PageExpectation::ValidContent(ref content) => {
                                            if content != "" {
                                                if raw_page_content.contains(content) {
                                                    info!("Got expected content: {} from URL: {}", content, page_url);
                                                } else {
                                                    error!("Failed to find content: {} from URL: {}", content, page_url);
                                                }
                                            } else {
                                                debug!("Skipped page content expectation for URL: {}", page_url);
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
