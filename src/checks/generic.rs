use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use ssl_expiration::SslExpiration;
use std::io::{Error, ErrorKind};
use std::time::Duration;

use crate::*;


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
        read_text_file(&name).and_then(|file_contents| {
            serde_json::from_str(&*file_contents)
                .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
        })
    }


    fn execute(&self) -> History {
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
                    .filter(|story| story.error.is_some())
                    .map(|story| {
                        if let Some(error) = &story.error {
                            format!("{}, ", error)
                        } else {
                            String::new()
                        }
                    })
                    .collect::<String>();

                debug!("Executing notification to channel: {}", &channel);
                notify_failure(webhook, channel, &failures);
            }
            (..) => {
                info!("Notifications not configured hence skipped…");
            }
        };
        history
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
