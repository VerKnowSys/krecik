use crate::{products::story::*, utilities, Config};
use actix::prelude::*;
use std::{collections::HashMap, fs, path::Path};


/// Notificator actor for Curl Multi bulk checks
#[derive(Debug, Copy, Clone)]
pub struct Notificator;


/// List of Check(s)
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Notify(pub Stories);


impl Handler<Notify> for Notificator {
    type Result = ();

    fn handle(&mut self, stories: Notify, _ctx: &mut Self::Context) -> Self::Result {
        let notifiers = Config::load().notifiers.unwrap_or_default();
        trace!("Defined notifiers: {:#?}", notifiers);

        let mut sorted_tuples = stories
            .0
            .iter()
            .map(|elem| {
                let error = elem.error.clone().unwrap().to_string();
                let notifier = elem.notifier.clone().unwrap_or_default();
                (error, notifier)
            })
            .collect::<Vec<(String, String)>>();
        sorted_tuples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // let's iterate over each string and count occurences
        // if there are 3 occurences - we should send notification about it:
        let mut failure_occurences = HashMap::new();
        for element in sorted_tuples {
            let existing_value = failure_occurences.entry(element).or_insert(0);
            *existing_value += 1;
        }
        debug!("Failure occurences: {:#?}", failure_occurences);

        let ok_message = Config::load()
            .ok_message
            .unwrap_or_else(|| String::from("All services are UP."));

        let previous_errors_with_webhooks = failure_occurences
            .iter()
            .filter(|&(_k, v)| *v == 2)
            .map(|(tuple, _v)| {
                let notifier = notifiers
                    .iter()
                    .find(|e| e.name == tuple.1)
                    .cloned()
                    .unwrap_or_default()
                    .slack_webhook;
                (format!("{}\n", ok_message), tuple.clone().1, notifier)
            })
            .collect::<Vec<(String, String, String)>>();

        let errors_with_webhooks = failure_occurences
            .iter()
            .filter(|&(_k, v)| *v == 3)
            .map(|(tuple, _v)| {
                let notifier = notifiers
                    .iter()
                    .find(|e| e.name == tuple.1)
                    .cloned()
                    .unwrap_or_default()
                    .slack_webhook;
                (format!("{}\n", tuple.clone().0), tuple.clone().1, notifier)
            })
            .collect::<Vec<(String, String, String)>>();

        // if errors_with_webhooks are empty and previous_errors_with_webhooks contains 2 error entries,
        // we can pick which one it was and send succes notification to it (after previos failure):
        if errors_with_webhooks.is_empty() {
            for (message, notifier_name, webhook) in previous_errors_with_webhooks {
                let last_notifications_file =
                    format!("/tmp/krecik-last-notification_{}", notifier_name);
                let last_notifications =
                    utilities::read_text_file(&last_notifications_file).unwrap_or_default();

                if last_notifications == message {
                    info!("Repeated OK notification skipped.");
                } else if Path::new(&format!("{}.after-failure", &last_notifications_file))
                    .exists()
                {
                    fs::remove_file(&format!("{}.after-failure", last_notifications_file))
                        .unwrap_or_default();
                    fs::remove_file(&last_notifications_file).unwrap_or_default();
                    utilities::write_append(
                        &last_notifications_file,
                        &format!("{:#?}", message),
                    );
                    warn!(
                        "Sending OK notification: '{}' to notifier id: {}, webhook: '{}'",
                        &message, &notifier_name, &webhook
                    );
                    // utilities::notify_success(&webhook, &ok_message);
                }
            }
        } else {
            for (message, notifier_name, webhook) in errors_with_webhooks.clone() {
                let last_notifications_file =
                    format!("/tmp/krecik-last-notification_{}", notifier_name);
                let last_notifications =
                    utilities::read_text_file(&last_notifications_file).unwrap_or_default();
                // create state that will be used by OK notification
                fs::copy(
                    &last_notifications_file,
                    &format!("{}.after-failure", last_notifications_file),
                )
                .unwrap_or_default();

                if last_notifications == message {
                    info!("Repeated FAIL notification skipped.");
                } else {
                    fs::remove_file(&last_notifications_file).unwrap_or_default();
                    utilities::write_append(&last_notifications_file, &message);
                    warn!(
                        "Sending FAIL notification: '{}' to notifier id: {}, webhook: '{}'",
                        &message, &notifier_name, &webhook
                    );
                    // utilities::notify_failure(&webhook, &ok_message);
                }
            }
        }
    }
}


impl Actor for Notificator {
    type Context = SyncContext<Self>;
}
