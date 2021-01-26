use actix::prelude::*;
use std::fs;

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
const STORIES_TO_KEEP_COUNT: usize = 60 * 12; // keep half a day of stories


impl Handler<ValidateResults> for ResultsWarden {
    type Result = Result<(), ()>;

    fn handle(&mut self, _msg: ValidateResults, _ctx: &mut Self::Context) -> Self::Result {
        info!("ResultsWarden validates results…");
        let stories_glob = "/tmp/krecik-history-*.json";
        let files_list = produce_list_absolute(&stories_glob)
            .iter()
            .rev()
            .take(STORIES_TO_VALIDATE_COUNT)
            .cloned()
            .collect::<Vec<String>>();
        if files_list.is_empty() {
            warn!("No results. Nothing to validate.");
            return Ok(());
        }

        debug!("Last stories file name: {}", &files_list[0]);
        let last_stories: Vec<Story> =
            serde_json::from_str(&read_text_file(&files_list[0]).unwrap_or_default())
                .unwrap_or(vec![]);
        let last_stories_errors = last_stories
            .iter()
            .filter(|entry| entry.error.is_some())
            .cloned()
            .collect::<Vec<Story>>();

        if files_list.len() < STORIES_TO_VALIDATE_COUNT {
            info!(
                "Less than {} stories available, skipping validation…",
                STORIES_TO_VALIDATE_COUNT
            );
        } else {
            info!(
                "Validating last stories from {} recent files: {:?}",
                STORIES_TO_VALIDATE_COUNT, files_list
            );

            let old_files_list = produce_list_absolute(&stories_glob)
                .iter()
                .rev()
                .skip(STORIES_TO_KEEP_COUNT)
                .cloned()
                .collect::<Vec<String>>();
            debug!("Wiping out old stories: {:?}", old_files_list);
            for old_file in old_files_list {
                fs::remove_file(&old_file).unwrap_or_default();
            }

            let previous_stories: Vec<Story> =
                serde_json::from_str(&read_text_file(&files_list[1]).unwrap_or_default())
                    .unwrap_or(vec![]);
            let previous_stories_errors = previous_stories
                .iter()
                .filter(|entry| entry.error.is_some())
                .cloned()
                .collect::<Vec<Story>>();

            let old_previous_stories: Vec<Story> =
                serde_json::from_str(&read_text_file(&files_list[2]).unwrap_or_default())
                    .unwrap_or(vec![]);
            let old_previous_stories_errors = old_previous_stories
                .iter()
                .filter(|entry| entry.error.is_some())
                .cloned()
                .collect::<Vec<Story>>();

            info!("[0]: {:?}", last_stories_errors);
            info!("[1]: {:?}", previous_stories_errors);
            info!("[2]: {:?}", old_previous_stories_errors);

            // TODO: add notification logic:
            // TODO: send success notification when previous_stories_errors or old_previous_stories_errors contain errors, and last_stories_errors is ok
            // TODO: send failure notification when last_stories_errors and previous_stories_errors and old_previous_stories_errors contain same error
        }

        // if an error is detected in last stories, run next check without a pause in-between:
        if last_stories_errors.is_empty() {
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
