use actix::prelude::*;

use crate::{
    products::story::Story,
    utilities::{produce_list_absolute, read_text_file},
};


/// ResultsWarden actor will check stories result and send alert notification if necessary
#[derive(Debug, Copy, Clone)]
pub struct ResultsWarden;


/// Validates results history
#[derive(Message, Debug, Clone, Copy)]
#[rtype(result = "Result<(), ()>")]
pub struct ValidateResults;

const STORIES_TO_VALIDATE_COUNT: usize = 3;


impl Handler<ValidateResults> for ResultsWarden {
    type Result = Result<(), ()>;

    fn handle(&mut self, _msg: ValidateResults, _ctx: &mut Self::Context) -> Self::Result {
        info!("ResultsWarden validates results…");

        let stories = "/tmp/krecik-history-*.json";
        let files_list = produce_list_absolute(&stories)
            .iter()
            .rev()
            .take(STORIES_TO_VALIDATE_COUNT)
            .cloned()
            .collect::<Vec<String>>();

        debug!("Last stories file name: {}", &files_list[0]);
        let last_stories: Vec<Story> =
            serde_json::from_str(&read_text_file(&files_list[0]).unwrap_or_default()).unwrap();

        if files_list.len() < STORIES_TO_VALIDATE_COUNT {
            debug!(
                "Less than {} stories available, skipping validation…",
                STORIES_TO_VALIDATE_COUNT
            );
        } else {
            info!(
                "Validating last stories from {} recent files: {:?}",
                STORIES_TO_VALIDATE_COUNT, files_list
            );
            // TODO: perform validation for files_list and send Slack notification if all 3 stories file have same error
        }

        // if an error is detected in last stories, run next check without a pause in-between:
        if last_stories
            .iter()
            .filter(|entry| entry.error.is_some())
            .collect::<Vec<_>>()
            .is_empty()
        {
            info!("No errors in last stories!");
            Ok(())
        } else {
            error!("There were errors in last stories!");
            Err(())
        }
    }
}


impl Actor for ResultsWarden {
    type Context = SyncContext<Self>;
}
