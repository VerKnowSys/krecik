use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};
use std::io::{Error, ErrorKind};

use crate::configuration::*;
use crate::utilities::*;
use crate::inputs::check::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;
use crate::products::unexpected::*;


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

        Ok(())
    }


}
