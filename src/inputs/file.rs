use std::io::{Error, ErrorKind};

use crate::configuration::*;
use crate::utilities::*;
use crate::inputs::check::*;
use crate::checks::page::*;
use crate::checks::domain::*;
use crate::products::expected::*;


/// NOTE: Pigeon (previous implementation) supported list of checks per file. TravMole will require each JSON to be separate file.
///       Decission is justified by lack of JSON comment ability, and other file-specific and sync troubles,
///       but also for future editing/ enable/ disable abilities that would be much more complicated with support of several checks per file.


#[derive(Debug, Clone, Serialize, Deserialize)]
/// FileCheck structure
pub struct FileCheck {

    /// Unique check name
    name: String,

    /// Domains to check
    domains: Option<Domains>,

    /// Pages to check
    pages: Option<Pages>,

    /// Slack Webhook
    alert_webhook: Option<String>,

    /// Slack alert channel
    alert_channel: Option<String>,

}


impl Checks<FileCheck> for FileCheck {


    /// Load check from JSON file
    fn load(name: &str) -> Result<FileCheck, Error> {
        Self::load_from(name, CHECKS_DIR)
    }


    /// Load check from JSON file by name
    fn load_from(name: &str, checks_dir: &str) -> Result<FileCheck, Error> {
        let check_file = format!("{}/{}.json", &checks_dir, &name);
        read_text_file(&check_file)
            .and_then(|file_contents| {
                serde_json::from_str(&file_contents.to_string())
                    .map_err(|err| Error::new(ErrorKind::Other, err.to_string()))
            })
    }


}
