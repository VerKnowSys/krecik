use crate::*;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// History is list of Stories
pub struct History(Stories);


impl History {
    /// New empty history
    pub fn empty() -> History {
        History(vec![])
    }


    /// New History with first element
    pub fn new(first: Story) -> History {
        History(vec![first])
    }


    /// New History with stories list
    pub fn new_from(stories: Stories) -> History {
        History(stories)
    }


    /// Stories extractor
    pub fn stories(&self) -> Stories {
        self.0.clone()
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
        History([self.0.clone(), vec![story]].concat())
    }


    /// Merge History with another History
    pub fn merge(&self, a_history: History) -> History {
        match a_history {
            History(stories) => History([self.0.clone(), stories].concat()),
        }
    }
}


/// Implement JSON serialization on .to_string():
impl ToString for History {
    fn to_string(&self) -> String {
        serde_json::to_string(&self.0).unwrap_or_else(|_| {
            String::from("{\"status\": \"History serialization failure\"}")
        })
    }
}
