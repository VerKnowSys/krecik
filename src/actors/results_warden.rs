use crate::{
    products::story::Story,
    utilities::{produce_list_absolute, read_text_file},
    Notificator, Notify, STORIES_TO_KEEP_COUNT, STORIES_TO_VALIDATE_COUNT,
};
use actix::prelude::*;
use std::fs;


/// ResultsWarden actor will check stories result and send alert notification if necessary
#[derive(Debug, Copy, Clone)]
pub struct ResultsWarden;


/// Validates results history
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct ValidateResults(pub Addr<Notificator>);


impl Handler<ValidateResults> for ResultsWarden {
    type Result = ();

    fn handle(&mut self, val: ValidateResults, _ctx: &mut Self::Context) -> Self::Result {
        info!("ResultsWarden validates results…");
        let stories_glob = "/tmp/krecik-history-*.json";
        let files_list = produce_list_absolute(stories_glob)
            .iter()
            .rev()
            .take(STORIES_TO_VALIDATE_COUNT)
            .cloned()
            .collect::<Vec<String>>();
        if files_list.is_empty() {
            warn!("No results. Nothing to validate.");
            return;
        }

        debug!("Last stories file name: {}", &files_list[0]);
        let last_stories: Vec<Story> =
            serde_json::from_str(&read_text_file(&files_list[0]).unwrap_or_default())
                .unwrap_or_default();
        if last_stories.is_empty() {
            warn!(
                "Last stories seems to be incomplete? Skipping validation until next iteration."
            );
            return;
        }
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
            debug!(
                "Validating last stories from {} recent files: {:?}",
                STORIES_TO_VALIDATE_COUNT, files_list
            );

            let old_files_list = produce_list_absolute(stories_glob)
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
                    .unwrap_or_default();
            let previous_stories_errors = previous_stories
                .iter()
                .filter(|entry| entry.error.is_some())
                .cloned()
                .collect::<Vec<Story>>();

            let old_previous_stories: Vec<Story> =
                serde_json::from_str(&read_text_file(&files_list[2]).unwrap_or_default())
                    .unwrap_or_default();
            let old_previous_stories_errors = old_previous_stories
                .iter()
                .filter(|entry| entry.error.is_some())
                .cloned()
                .collect::<Vec<Story>>();

            debug!("Error stories:");
            debug!("[0]: {:?}", last_stories_errors);
            debug!("[1]: {:?}", previous_stories_errors);
            debug!("[2]: {:?}", old_previous_stories_errors);

            let notifier = val.0;
            notifier.do_send(Notify(
                [
                    last_stories_errors,
                    previous_stories_errors,
                    old_previous_stories_errors,
                ]
                .concat(),
            ));
        }
    }
}


impl Actor for ResultsWarden {
    type Context = SyncContext<Self>;
}
