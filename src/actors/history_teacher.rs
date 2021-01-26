use crate::{products::story::Story, utilities};
use actix::prelude::*;
use chrono::*;


/// HistoryTeacher actor stores check results to a json file
#[derive(Debug, Copy, Clone)]
pub struct HistoryTeacher;


/// List of result stories
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<(), ()>")]
pub struct Results(pub Vec<Story>);


impl Handler<Results> for HistoryTeacher {
    type Result = Result<(), ()>;

    fn handle(&mut self, history: Results, _ctx: &mut Self::Context) -> Self::Result {
        let stories_listof_json = history
            .0
            .iter()
            .map(|story| story.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let history_json = format!("[{}]", stories_listof_json);
        let timestamp = Local::now().to_rfc3339();
        let stories_output = format!("/tmp/krecik-history-{}.json", timestamp);
        info!("Storing check result stories to file: {}", stories_output);
        utilities::write_append(&stories_output, &history_json);
        Ok(())
    }
}


impl Actor for HistoryTeacher {
    type Context = SyncContext<Self>;
}
