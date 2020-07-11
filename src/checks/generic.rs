use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;


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
                        notify_success(webhook, channel, "All services are UP again.");
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
                    if send_notification.is_some() {
                        notify_failure(webhook, channel, &failures);
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
