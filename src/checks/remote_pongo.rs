use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;

use crate::configuration::*;
use crate::utilities::*;
use crate::checks::*;
use crate::checks::generic::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::products::history::*;
use crate::mappers::pongo::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Map Remote fields/values mapper structure to GenCheck:
pub struct PongoRemoteMapper {

    /// Resource URL
    pub url: String,

}


impl PongoRemoteMapper {

    fn empty() -> Self {
        PongoRemoteMapper {
            url: "".to_string(),
        }
    }

}


impl Checks<GenCheck> for PongoHost {


    fn load(remote_file_name: &str) -> Result<GenCheck, Error> {
        let mapper: PongoRemoteMapper
            = read_text_file(&remote_file_name)
                .and_then(|file_contents| {
                    serde_json::from_str(&file_contents)
                        .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
                })
                .unwrap_or_else(|_| PongoRemoteMapper::empty());

        debug!("Loaded mapper: {:#?}", mapper);
        let mut easy = Easy2::new(Collector(Vec::new()));
        easy.get(true).unwrap();
        easy.url(&mapper.url).unwrap();
        easy.perform().unwrap();
        let contents = easy.get_ref();
        let remote_raw = String::from_utf8_lossy(&contents.0);
        debug!("PongoRemoteMapper::load from URL: {}, JSON RAW length: {}",
               mapper.url.replace(r"token=.*", "token=**masked**"),
               &remote_raw.len());

        // now use default Pongo structure defined as default for PongoRemoteMapper
        let pongo_hosts: PongoHosts
            = serde_json::from_str(&remote_raw)
                .unwrap_or_default();
        let pongo_checks
            = pongo_hosts
                .clone()
                .into_iter()
                .flat_map(|host| {
                    host
                        .data
                        .host
                        .vhosts
                        .and_then(|vhosts| {
                            vhosts
                                .iter()
                                .filter(|vhost| !vhost.contains("*.")) // filter out wildcard domains
                                .map(|vhost| {
                                    Some(
                                        Page {
                                            url: format!("{}{}", CHECK_DEFAULT_PROTOCOL, vhost),
                                            expects: Some(Self::default_page_expectations()),
                                            options: None,
                                        }
                                    )
                                })
                                .collect::<Option<Pages>>()
                        })
                        .unwrap_or_default()
                })
                .collect();
        let domain_checks
            = pongo_hosts
                .clone()
                .into_iter()
                .flat_map(|host| {
                    host
                        .data
                        .host
                        .vhosts
                        .and_then(|vhosts| {
                            vhosts
                                .iter()
                                .filter(|vhost| !vhost.contains("*.")) // filter out wildcard domains
                                .map(|vhost| {
                                    Some(
                                        Domain {
                                            name: vhost.to_string(),
                                            expects: Some(Self::default_domain_expectations()),
                                        }
                                    )
                                })
                                .collect::<Option<Domains>>()
                        })
                        .unwrap_or_default()
                })
                .collect();
        debug!("Pongo hosts: {:#?}", pongo_hosts);
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
