use hyper::*;
use mime::APPLICATION_JSON;
use chrono::Local;
use gotham::{handler::IntoResponse, state::State, helpers::http::response::create_response};

use crate::products::expected::*;
use crate::products::unexpected::*;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Story holds errornous state
pub struct Story {

    /// Story - timestamp
    pub timestamp: String,

    /// Story - failure count
    pub count: u64,

    /// Story - message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Story - keep history of unexpected results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Unexpected>,

}


impl Story {


    /// New story
    pub fn new(message: Option<String>) -> Story {
        let local_now = Local::now();
        Story {
            count: 1,
            timestamp: local_now.to_rfc3339(),
            message,
            error: None,
        }
    }


    /// New error story
    pub fn new_error(error: Option<Unexpected>) -> Story {
        let local_now = Local::now();
        Story {
            count: 1,
            timestamp: local_now.to_rfc3339(),
            message: None,
            error,
        }
    }


}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// History is list of Stories
pub struct History(Vec<Story>);


impl History {


    /// New empty history
    pub fn empty() -> History {
        History(vec!())
    }


    /// New History
    pub fn new(first: Story) -> History {
        History(vec!(first))
    }


    /// Head of the History - first element added
    pub fn head(&self) -> Story {
        self.0[0].clone()
    }


    /// History length
    pub fn length(&self) -> usize {
        self.0.len()
    }


    /// Append Story to History
    pub fn append(&self, story: Story) -> History {
        History([self.0.clone(), vec!(story)].concat())
    }


    /// Merge History with another History
    pub fn merge(&self, a_history: History) -> History {
        let mut history = History(self.0.clone());
        match a_history {
            History(stories) => {
                for story in stories {
                    history = history.append(story);
                }
            }
        }
        history
    }

}


/// Implement Gotham response for History:
impl IntoResponse for History {
    fn into_response(self, state: &State) -> Response<Body> {
        create_response(
            state,
            StatusCode::OK,
            APPLICATION_JSON,
            serde_json::to_string(&self.0)
                .unwrap_or_else(|_| String::from("{\"status\": \"History serialization failure\"}")),
        )
    }
}
