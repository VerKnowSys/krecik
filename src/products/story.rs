use chrono::Local;

use crate::products::expected::*;
use crate::products::unexpected::*;


/// Alias Type for Vec<Story>
pub type Stories = Vec<Story>;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Story holds errornous state
pub struct Story {
    /// Story - timestamp
    pub timestamp: String,

    /// Story - failure count
    pub count: u64,

    /// Story - success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<Expected>,

    /// Story - keep history of unexpected results (failures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Unexpected>,
}


impl Story {
    /// New success-story
    pub fn success(success: Expected) -> Story {
        Story {
            count: 1,
            timestamp: Local::now().to_rfc3339(),
            success: Some(success),
            error: None,
        }
    }


    /// New error-story
    pub fn error(error: Unexpected) -> Story {
        Story {
            count: 1,
            timestamp: Local::now().to_rfc3339(),
            success: None,
            error: Some(error),
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
