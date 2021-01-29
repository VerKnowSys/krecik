use actix::prelude::*;
use chrono::Local;

use crate::*;


/// Alias Type for Vec<Story>
pub type Stories = Vec<Story>;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Message)]
#[rtype(result = "Story")]
/// Story holds the story of a check. It can be either: success, error or minor
pub struct Story {
    /// Story - timestamp
    pub timestamp: String,

    /// Story - success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<Expected>,

    /// Story - minor failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<UnexpectedMinor>,

    /// Story - keep history of unexpected results (failures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Unexpected>,

    /// Notifier to use if notification action is necessary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifier: Option<String>,
}


impl Story {
    /// New success-story
    pub fn success(success: Expected, notifier: Option<String>) -> Story {
        Story {
            timestamp: Local::now().to_rfc3339(),
            success: Some(success),
            minor: None,
            error: None,
            notifier,
        }
    }


    /// New error-story
    pub fn error(error: Unexpected, notifier: Option<String>) -> Story {
        Story {
            timestamp: Local::now().to_rfc3339(),
            success: None,
            minor: None,
            error: Some(error),
            notifier,
        }
    }


    /// New minor-failure-story (not notified)
    pub fn minor(minor: UnexpectedMinor) -> Story {
        Story {
            timestamp: Local::now().to_rfc3339(),
            success: None,
            minor: Some(minor),
            error: None,
            notifier: None,
        }
    }
}


/// Implement JSON serialization on .to_string():
impl ToString for Story {
    fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .unwrap_or_else(|_| String::from("{\"status\": \"Story serialization failure\"}"))
    }
}
