use crate::*;
use curl::easy::Easy2;
use rayon::prelude::*;
use regex::Regex;
use std::{
    fs,
    io::{Error, ErrorKind},
    path::Path,
};


/// Read domain checks from pongo mapper
pub fn get_domain_checks(pongo_mapper: String) -> Check {
    let mapper = read_pongo_mapper(&pongo_mapper);
    let domain_checks = get_pongo_hosts(&mapper.url)
        .into_par_iter()
        .flat_map(|check| collect_pongo_domains(&check))
        .collect();
    Check {
        domains: Some(domain_checks),

        // pass alert webhook and channel from mapper to the checks
        alert_webhook: mapper.alert_webhook,
        alert_channel: mapper.alert_channel,
        ..Check::default()
    }
}


/// Read page checks from pongo mapper
pub fn get_page_checks(pongo_mapper: String) -> Check {
    let mapper = read_pongo_mapper(&pongo_mapper);
    let pongo_checks = get_pongo_hosts(&mapper.url)
        .into_par_iter()
        .flat_map(|check| collect_pongo_hosts(&check, &mapper))
        .collect();

    Check {
        pages: Some(pongo_checks),

        // pass alert webhook and channel from mapper to the checks
        alert_webhook: mapper.alert_webhook,
        alert_channel: mapper.alert_channel,
        ..Check::default()
    }
}


/// Collect pongo domain check by host
pub fn collect_pongo_domains(check: &PongoCheck) -> Vec<Domain> {
    check
        .data
        .host
        .clone()
        .unwrap_or_default()
        .vhosts
        .and_then(|vhosts| {
            vhosts
                .par_iter()
                .filter(|vhost| !vhost.starts_with("*.")) // filter out wildcard domains
                .map(|vhost| {
                    Some(Domain {
                        name: vhost.to_string(),
                        expects: default_domain_expectations(),
                    })
                })
                .collect::<Option<Domains>>()
        })
        .unwrap_or_default()
}


/// Collect pongo page checks by host
pub fn collect_pongo_hosts(check: &PongoCheck, mapper: &PongoRemoteMapper) -> Vec<Page> {
    let ams = check.clone().data.ams.unwrap_or_default();
    let active = check.active.unwrap_or(false);
    let client = check.clone().client.unwrap_or_default();
    let options = check.clone().options;
    [
        // merge two lists for URLs: "vhosts" and "showrooms":
        check
            .clone()
            .data
            .host
            .unwrap_or_default()
            .vhosts
            .and_then(|vhosts| {
                vhosts
                    .par_iter()
                    .filter(|vhost| {
                        !vhost.starts_with("*.")
                            && vhost.contains(
                                &mapper.only_vhost_contains.clone().unwrap_or_default(),
                            )
                    }) // filter out wildcard domains and pick only these matching value of only_vhost_contains field
                    .map(|vhost| {
                        if active {
                            Some(Page {
                                url: format!("{}{}/{}/", CHECK_DEFAULT_PROTOCOL, vhost, ams),
                                expects: pongo_page_expectations(),
                                options: options.clone(),
                            })
                        } else {
                            debug!("Skipping not active client: {}", &client);
                            None
                        }
                    })
                    .collect::<Option<Pages>>()
            })
            .unwrap_or_default(),
        check
            .data
            .clone()
            .host
            .unwrap_or_default()
            .showroom_urls
            .and_then(|showrooms| {
                showrooms
                    .par_iter()
                    .map(|vhost| {
                        if active {
                            Some(Page {
                                url: vhost.to_string(),
                                expects: showroom_page_expectations(),
                                options: None,
                            })
                        } else {
                            debug!("Skipping not active client: {}", &client);
                            None
                        }
                    })
                    .collect::<Option<Pages>>()
            })
            .unwrap_or_default(),
    ]
    .concat()
}


/// Read Pongo mapper object
pub fn read_pongo_mapper(pongo_mapper: &str) -> PongoRemoteMapper {
    read_text_file(&pongo_mapper)
        .and_then(|file_contents| {
            serde_json::from_str(&file_contents)
                .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
        })
        .unwrap_or_default()
}


/// Pongo remote read utility
pub fn get_pongo_hosts(url: &str) -> PongoChecks {
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap_or_default();
    easy.url(&url).unwrap_or_default();
    easy.perform().unwrap_or_default();
    let contents = easy.get_ref();
    let remote_raw = String::from_utf8_lossy(&contents.0);
    serde_json::from_str(&remote_raw)
        .map_err(|err| {
            error!(
                "Failed to parse Pongo input: {:#?}. Caused by: {:?}",
                remote_raw, err
            )
        })
        .unwrap_or_default()
}


/// Provide pongo page expectations:
pub fn pongo_page_expectations() -> PageExpectations {
    vec![
        PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE),
        // PageExpectation::ValidLength(CHECK_HTTP_MINIMUM_LENGHT),
        PageExpectation::ValidAddress("https://".to_string()),
        PageExpectation::ValidContent("SIGN IN".to_string()),
    ]
}


/// Provide pongo showroom page expectations:
pub fn showroom_page_expectations() -> PageExpectations {
    vec![
        PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE),
        // PageExpectation::ValidLength(CHECK_HTTP_MINIMUM_LENGHT),
        PageExpectation::ValidAddress("https://".to_string()),
        PageExpectation::ValidContent("API: 'https://".to_string()),
    ]
}


//
// Data structures based on private Centra API, called "Pongo":
//


/// List of Pongo checks
pub type PongoChecks = Vec<PongoCheck>;


/// Remote structure that will be loaded as GenCheck:
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PongoCheck {
    /// Client data:
    pub data: PongoHostData,

    /// Client name:
    pub client: Option<String>,

    /// Client is active?:
    pub active: Option<bool>,

    /// Curl options:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PageOptions>,

    /// Slack Webhook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_webhook: Option<String>,

    /// Slack alert channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_channel: Option<String>,

    /// Domains to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Domains>,

    /// Pages to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Pages>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHostData {
    /// Host inner object:
    pub host: Option<PongoHostDetails>,

    /// Client env:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,

    /// Client ams:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ams: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHostDetails {
    /// List of virtual hosts of client:
    pub vhosts: Option<Vec<String>>,

    /// Showroom urls of client:
    pub showroom_urls: Option<Vec<String>>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Map Remote fields/values mapper structure to GenCheck:
pub struct PongoRemoteMapper {
    /// Resource URL
    pub url: String,

    /// Check AMS only for specified subdomain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_vhost_contains: Option<String>,

    /// Slack Webhook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_webhook: Option<String>,

    /// Slack alert channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_channel: Option<String>,
}


impl Checks<PongoCheck> for PongoCheck {
    fn load(remote_file_name: &str) -> Result<PongoCheck, Error> {
        let mapper: PongoRemoteMapper = read_text_file(&remote_file_name)
            .and_then(|file_contents| {
                serde_json::from_str(&file_contents)
                    .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
            })
            .unwrap_or_default();

        let mut easy = Easy2::new(Collector(Vec::new()));
        easy.get(true).unwrap_or_default();
        easy.url(&mapper.url).unwrap_or_default();
        easy.perform().unwrap_or_default();
        let contents = easy.get_ref();
        let remote_raw = String::from_utf8_lossy(&contents.0);
        debug!(
            "PongoRemoteMapper::load REMOTE-JSON length: {}",
            &remote_raw.len().to_string().cyan()
        );

        // now use default Pongo structure defined as default for PongoRemoteMapper
        let pongo_hosts: PongoChecks = serde_json::from_str(&remote_raw)
            .map_err(|err| error!("Failed to parse Pongo input: {:#?}", err))
            .unwrap_or_default();

        debug!("Pongo hosts: {:#?}", &pongo_hosts);
        let pongo_checks = pongo_hosts
            .clone()
            .into_par_iter()
            .flat_map(|host| {
                let ams = host.clone().data.ams.unwrap_or_default();
                let active = host.active.unwrap_or_else(|| false);
                let client = host.clone().client.unwrap_or_default();
                let options = host.clone().options;

                let pongo_private_token = Regex::new(r"\?token=[A-Za-z0-9_-]*").unwrap();
                let safe_url = pongo_private_token.replace(&mapper.url, "[[token-masked]]");
                debug!(
                    "Pongo: URL: {}, CLIENT: {}, AMS: {}. ACTIVE: {}",
                    &safe_url.cyan(),
                    &client.cyan(),
                    &ams.cyan(),
                    format!("{}", active).cyan()
                );
                [
                    // merge two lists for URLs: "vhosts" and "showrooms":
                    host.clone()
                        .data
                        .host
                        .unwrap_or_default()
                        .vhosts
                        .and_then(|vhosts| {
                            vhosts
                                .par_iter()
                                .filter(|vhost| {
                                    !vhost.starts_with("*.")
                                        && vhost.contains(
                                            &mapper
                                                .only_vhost_contains
                                                .clone()
                                                .unwrap_or_default(),
                                        )
                                }) // filter out wildcard domains and pick only these matching value of only_vhost_contains field
                                .map(|vhost| {
                                    if active {
                                        Some(Page {
                                            url: format!(
                                                "{}{}/{}/",
                                                CHECK_DEFAULT_PROTOCOL, vhost, ams
                                            ),
                                            expects: pongo_page_expectations(),
                                            options: options.clone(),
                                        })
                                    } else {
                                        debug!("Skipping not active client: {}", &client);
                                        None
                                    }
                                })
                                .collect::<Option<Pages>>()
                        })
                        .unwrap_or_default(),
                    host.data
                        .host
                        .unwrap_or_default()
                        .showroom_urls
                        .and_then(|showrooms| {
                            showrooms
                                .par_iter()
                                .map(|vhost| {
                                    if active {
                                        Some(Page {
                                            url: vhost.to_string(),
                                            expects: showroom_page_expectations(),
                                            options: None,
                                        })
                                    } else {
                                        debug!("Skipping not active client: {}", &client);
                                        None
                                    }
                                })
                                .collect::<Option<Pages>>()
                        })
                        .unwrap_or_default(),
                ]
                .concat()
            })
            .collect();
        let domain_checks = pongo_hosts
            .into_par_iter()
            .flat_map(|host| {
                host.data
                    .host
                    .unwrap_or_default()
                    .vhosts
                    .and_then(|vhosts| {
                        vhosts
                            .par_iter()
                            .filter(|vhost| !vhost.starts_with("*.")) // filter out wildcard domains
                            .map(|vhost| {
                                Some(Domain {
                                    name: vhost.to_string(),
                                    expects: default_domain_expectations(),
                                })
                            })
                            .collect::<Option<Domains>>()
                    })
                    .unwrap_or_default()
            })
            .collect();
        Ok(PongoCheck {
            pages: Some(pongo_checks),
            domains: Some(domain_checks),

            // pass alert webhook and channel from mapper to the checks
            alert_webhook: mapper.alert_webhook,
            alert_channel: mapper.alert_channel,
            ..PongoCheck::default()
        })
    }


    fn execute(&self, execution_name: &str) -> History {
        let history = History::new_from(
            [
                Self::check_pages(self.pages.clone()).stories(),
                Self::check_domains(self.domains.clone()).stories(),
            ]
            .concat(),
        );
        match (&self.alert_webhook, &self.alert_channel) {
            (Some(webhook), Some(channel)) => {
                let failures = history
                    .stories()
                    .iter()
                    .filter_map(|story| {
                        if let Some(error) = &story.error {
                            Some(format!("{}\n", error))
                        } else {
                            None
                        }
                    })
                    .collect::<String>();

                let failures_state_file =
                    &format!("{}-{}", DEFAULT_FAILURES_STATE_FILE, execution_name);
                debug!("Failures state file: {}", failures_state_file);
                debug!("FAILURES: {:?}", failures);
                if failures.is_empty() {
                    if Path::new(failures_state_file).exists() {
                        debug!(
                            "No more failures! Removing failures log file and notifying that failures are gone"
                        );
                        fs::remove_file(failures_state_file).unwrap_or_default();
                        notify_success(
                            webhook,
                            channel,
                            &format!("All services are UP again ({}).\n", &execution_name),
                        );
                    } else {
                        debug!("All services are OK! No notification sent");
                    }
                } else {
                    // there are errors:
                    let file_entries = read_text_file(failures_state_file).unwrap_or_default();

                    let send_notification = failures.split('\n').find(|fail| {
                        if !file_entries.contains(fail) {
                            write_append(failures_state_file, &fail.to_string());
                            true
                        } else {
                            false
                        }
                    });
                    // send notification only for new error that's not present in failure state
                    let failures_to_notify = failures
                        .split('\n')
                        .filter(|fail| !file_entries.contains(fail))
                        .map(|fail| format!("{}\n", fail))
                        .collect::<String>();

                    if send_notification.is_some() {
                        notify_failure(webhook, channel, &failures_to_notify);
                    }
                }
            }
            (..) => {
                info!("Notifications not configured hence skipped…");
            }
        };
        history
    }
}


/// Implement JSON serialization on .to_string():
impl ToString for PongoRemoteMapper {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| {
            String::from("{\"status\": \"PongoRemoteMapper serialization failure\"}")
        })
    }
}
