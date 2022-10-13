use crate::{
    checks::{domain::*, page::*},
    configuration::*,
    products::expected::{PageExpectation, PageExpectations},
    utilities::read_text_file,
    *,
};

use curl::easy::Easy2;
use rayon::prelude::*;
use std::io::{Error, ErrorKind};


/// Collect pongo domain check by host
#[instrument]
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
#[instrument]
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
    ]
    .concat()
}


/// Read Pongo mapper object
#[instrument]
pub fn read_pongo_mapper(pongo_mapper: &str) -> PongoRemoteMapper {
    read_text_file(pongo_mapper)
        .and_then(|file_contents| {
            serde_json::from_str(&file_contents)
                .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
        })
        .unwrap_or_default()
}


/// Read checks from Pongo remote
#[instrument]
pub fn get_pongo_checks(url: &str) -> PongoChecks {
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap_or_default();
    easy.url(url).unwrap_or_default();
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
#[instrument]
pub fn pongo_page_expectations() -> PageExpectations {
    vec![
        PageExpectation::ValidCode(CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE),
        // PageExpectation::ValidLength(CHECK_HTTP_MINIMUM_LENGHT),
        PageExpectation::ValidAddress("https://".to_string()),
        PageExpectation::ValidContent("SIGN IN".to_string()),
    ]
}


/// Provide pongo showroom page expectations:
#[instrument]
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

    /// Notifier id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifier: Option<String>,

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

    /// Notifier id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifier: Option<String>,
}


/// Implement JSON serialization on .to_string():
impl ToString for PongoRemoteMapper {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| {
            String::from("{\"status\": \"PongoRemoteMapper serialization failure\"}")
        })
    }
}
