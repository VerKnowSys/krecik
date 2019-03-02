use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;
use rayon::prelude::*;
use regex::Regex;

use crate::configuration::*;
use crate::utilities::*;
use crate::checks::*;
use crate::checks::generic::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::products::history::*;


//
// Data structures based on private Centra API, called "Pongo":
//


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHost {

    /// Domains to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Domains>,

    /// Pages to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Pages>,

    /// Updated at:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// Client name:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,

    /// Client is active?:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Client data:
    pub data: PongoHostData,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHostData {

    /// Client name:
    pub client: Option<String>,

    /// Client application environment:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,

    /// Client application ams name:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ams: Option<String>,

    /// Client main host name:
    pub host: PongoHostDetails,

    /// Client report:
    pub report: PongoReport,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoReport {

    /// Application modules enabled:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<String>>,

    /// Application processes:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processes: Option<String>,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Remote structure that will be loaded as GenCheck:
pub struct PongoHostDetails {

    /// Host IPv4:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    /// Primary host name:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_vhost: Option<String>,

    /// List of virtual hosts of client:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vhosts: Option<Vec<String>>,

    /// Backend SSHD port of client:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_port: Option<String>,

    /// Showroom urls of client:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub showroom_urls: Option<Vec<String>>,

}


/// PongoHosts collection type
pub type PongoHosts = Vec<PongoHost>;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Map Remote fields/values mapper structure to GenCheck:
pub struct PongoRemoteMapper {

    /// Resource URL
    pub url: String,


    /// Check AMS only for specified subdomain
    pub only_vhost_contains: String,

}


impl Checks<GenCheck> for PongoHost {


    fn load(remote_file_name: &str) -> Result<GenCheck, Error> {
        let mapper: PongoRemoteMapper
            = read_text_file(&remote_file_name)
                .and_then(|file_contents| {
                    serde_json::from_str(&file_contents)
                        .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
                })
                .unwrap_or_default();

        let mut easy = Easy2::new(Collector(Vec::new()));
        easy.get(true).unwrap_or_default();
        easy.url(&mapper.url).unwrap_or_default();
        easy.perform().unwrap_or_default();
        let contents = easy.get_ref();
        let remote_raw = String::from_utf8_lossy(&contents.0);
        debug!("PongoRemoteMapper::load REMOTE-JSON length: {}", &remote_raw.len().to_string().cyan());

        // now use default Pongo structure defined as default for PongoRemoteMapper
        let pongo_hosts: PongoHosts = serde_json::from_str(&remote_raw).unwrap_or_default();
        let pongo_checks
            = pongo_hosts
                .clone()
                .into_par_iter()
                .flat_map(|host| {
                    let ams = host.data.ams.unwrap_or_default();
                    let active = host.active.unwrap_or_else(|| false);
                    let client = host.client.unwrap_or_default();

                    let pongo_private_token = Regex::new(r"\?token=[A-Za-z0-9_-]*").unwrap();
                    let safe_url = pongo_private_token.replace(&mapper.url, "*[token-masked]*");
                    debug!("Pongo: URL: {}, CLIENT: {}, AMS: {}. ACTIVE: {}",
                           &safe_url.cyan(), &client.cyan(), &ams.cyan(), format!("{}", active).cyan());
                    [ // merge two lists for URLs: "vhosts" and "showrooms":
                        host
                            .data
                            .host
                            .vhosts
                            .and_then(|vhosts|
                                vhosts
                                    .par_iter()
                                    .filter(|vhost| !vhost.starts_with("*.") && vhost.contains(&mapper.only_vhost_contains)) // filter out wildcard domains and pick only these matching value of only_vhost_contains field
                                    .map(|vhost|
                                        if active {
                                            Some(
                                                Page {
                                                    url: format!("{}{}/{}", CHECK_DEFAULT_PROTOCOL, vhost, ams),
                                                    expects: default_page_expectations(),
                                                    options: None,
                                                }
                                            )
                                        } else {
                                            debug!("Skipping not active client: {}", &client);
                                            None
                                        }
                                    })
                                    .collect::<Option<Pages>>()
                            })
                            .unwrap_or_default(),

                        host
                            .data
                            .host
                            .showroom_urls
                            .and_then(|showrooms| {
                                showrooms
                                    .par_iter()
                                    .map(|vhost| {
                                        if active {
                                            Some(
                                                Page {
                                                    url: vhost.to_string(),
                                                    expects: PageExpectations::default(),
                                                    options: None,
                                                }
                                            )
                                        } else {
                                            debug!("Skipping not active client: {}", &client);
                                            None
                                        }
                                    })
                                    .collect::<Option<Pages>>()
                            })
                            .unwrap_or_default()

                    ].concat()
                })
                .collect();
        let domain_checks
            = pongo_hosts
                .clone()
                .into_par_iter()
                .flat_map(|host| {
                    host
                        .data
                        .host
                        .vhosts
                        .and_then(|vhosts|
                            vhosts
                                .par_iter()
                                .filter(|vhost| !vhost.starts_with("*.")) // filter out wildcard domains
                                .map(|vhost|
                                    Some(
                                        Domain {
                                            name: vhost.to_string(),
                                            expects: default_domain_expectations(),
                                        }
                                    )
                                })
                                .collect::<Option<Domains>>()
                        })
                        .unwrap_or_default()
                })
                .collect();
        // debug!("Pongo hosts: {:#?}", pongo_hosts);
        debug!("Pongo domains: {:#?}", domain_checks);
        debug!("Pongo pongo_checks: {:#?}", pongo_checks);

        Ok(
            GenCheck {
                pages: Some(pongo_checks),
                domains: Some(domain_checks),

                .. GenCheck::default()
            }
        )
    }


    fn execute(&self) -> History {
        History::new_from(
            [
                Self::check_pages(self.pages.clone()).stories(),
                Self::check_domains(self.domains.clone()).stories(),
            ].concat()
        )
    }


}


/// Implement JSON serialization on .to_string():
impl ToString for PongoRemoteMapper {
    fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .unwrap_or_else(|_| String::from("{\"status\": \"PongoRemoteMapper serialization failure\"}"))
    }
}


/// Implement default for PongoRemoteMapper:
impl Default for PongoRemoteMapper {
    fn default() -> Self {
        PongoRemoteMapper {
            url: "".to_string(),
            only_vhost_contains: "".to_string(),
        }
    }
}
