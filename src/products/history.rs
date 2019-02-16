use hyper::*;
use mime::APPLICATION_JSON;
use chrono::Local;
use gotham::{handler::IntoResponse, state::State, helpers::http::response::create_response};

use crate::products::expected::*;
use crate::products::unexpected::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Story holds errornous state
pub struct Story {

    /// Story - timestamp
    timestamp: i64,

    /// Story - failure count
    count: u64,

    /// Story - message
    message: Option<String>,

    /// Story - keep history of unexpected results
    error: Option<Unexpected>,

}


impl Story {


    /// New story
    pub fn new(message: Option<String>) -> Story {
        Story {
            count: 1,
            timestamp: Local::now().timestamp(),
            message,
            error: None,
        }
    }


    /// New error story
    pub fn new_error(error: Option<Unexpected>) -> Story {
        Story {
            count: 1,
            timestamp: Local::now().timestamp(),
            message: None,
            error,
        }
    }


}


#[derive(Debug, Clone, Serialize, Deserialize)]
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


    /// History length
    pub fn length(&self) -> usize {
        self.0.len()
    }


    /// Append Story to History
    pub fn append(&self, story: Story) -> History {
        History([self.0.clone(), vec!(story)].concat())
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
