use actix::prelude::*;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

use crate::*;


#[derive(Debug, Clone, Serialize, Deserialize, Default, Message)]
#[rtype(result = "Result<Stories, Stories>")]
/// Generic Check structure:
pub struct Check {
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

/// Alias type for list of checks
pub type CheckList = Vec<Check>;


#[deprecated(since = "0.9.0")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
                .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
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
