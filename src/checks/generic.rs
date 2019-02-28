use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};
use std::io::{Error, ErrorKind};
use std::time::Duration;

use crate::configuration::*;
use crate::utilities::*;
use crate::checks::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::products::history::*;


/// NOTE: Pigeon (previous implementation) supported list of checks per file. TravMole will require each JSON to be separate file.
///       Decission is justified by lack of JSON comment ability, and other file-specific and sync troubles,
///       but also for future editing/ enable/ disable abilities that would be much more complicated with support of several checks per file.


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Generic Check structure:
pub struct GenCheck {

    /// Domains to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Domains>,

    /// Pages to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Pages>,

    /// Slack Webhook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_webhook: Option<String>,

    /// Slack alert channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_channel: Option<String>,

}


impl Checks<GenCheck> for GenCheck {


    fn load(name: &str) -> Result<GenCheck, Error> {
        read_text_file(&name)
            .and_then(|file_contents| {
                serde_json::from_str(&*file_contents)
                    .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
            })
    }


    fn execute(&self) -> History {
        History::new_from(
            [
                GenCheck::check_pages(self.pages.clone()).stories(),
                GenCheck::check_domains(self.domains.clone()).stories(),
            ].concat()
        )
    }


}


impl Default for GenCheck {
    fn default() -> GenCheck {
        GenCheck {
            pages: None,
            domains: None,
            alert_channel: None,
            alert_webhook: None,
        }
    }
}
