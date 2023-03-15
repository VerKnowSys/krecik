use crate::*;
use std::{
    io::{Error, ErrorKind},
    path::Path,
};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Dynamic configuration read on demand by Krecik
pub struct Config {
    /// Absolute path to Krecik directory where "checks" are located
    pub krecik_root: Option<String>,

    /// Notification message when all checks are fine
    pub ok_message: Option<String>,

    /// List of named notifiers
    pub notifiers: Option<Vec<Notifiers>>,

    /// Success emoji used for notifications
    pub success_emoji: Option<String>,

    /// Failure emoji used for notifications
    pub failure_emoji: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Defines a notifier used later by notifications to send alerts
pub struct Notifiers {
    /// Notifier unique name
    pub name: String,

    // TODO: add more webhook types other than just Slack
    /// Notifier slack webhook
    pub slack_webhook: String,
}


impl Config {
    /// Load Krecik configuration file
    #[instrument]
    pub fn load() -> Config {
        let config_paths = [
            "/etc/krecik/krecik.conf",
            "/Services/Krecik/service.conf",
            "/Projects/krecik/krecik.conf",
            "krecik.conf",
        ];
        let config: String = config_paths
            .iter()
            .filter(|file| Path::new(file).exists())
            .take(1)
            .cloned()
            .collect();
        read_text_file(&config)
            .and_then(|file_contents| {
                serde_json::from_str(&file_contents).map_err(|err| {
                    let config_error = Error::new(ErrorKind::InvalidInput, err.to_string());
                    error!(
                        "Configuration error: {} in file: {}",
                        err.to_string(),
                        config
                    );
                    config_error
                })
            })
            .unwrap_or_default()
    }
}
