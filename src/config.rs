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

    /// Log output from Krecik-server
    pub log_file: Option<String>,

    /// Log level for Krecik-server
    pub log_level: Option<String>,

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
                serde_json::from_str(&*file_contents).map_err(|err| {
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

    /// Get LevelFilter (log level) from configuration
    pub fn get_log_level(&self) -> LevelFilter {
        let level = self.log_level.clone().unwrap_or_default();
        match &level[..] {
            "OFF" => LevelFilter::Off,
            "ERROR" => LevelFilter::Error,
            "WARN" => LevelFilter::Warn,
            "INFO" => LevelFilter::Info,
            "DEBUG" => LevelFilter::Debug,
            "TRACE" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    }
}
