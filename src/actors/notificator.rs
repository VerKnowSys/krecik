use crate::{products::story::*, utilities, Config};
use actix::prelude::*;
use std::{collections::HashMap, fs};


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
        // TODO: read ok_message from confiruation:
        let ok_message = "All services are UP.".to_string();
        let notification_contents = {
            let mut sorted_strings = stories
                .0
                .iter()
                .map(|elem| {
                    if let Some(error) = elem.error.clone() {
                        format!("{}\n", error.to_string())
                    } else {
                        "\n".to_string()
                    }
                })
                .collect::<Vec<String>>();
            sorted_strings.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // let's iterate over each string and count occurences
            // if there are 3 occurences - we should send notification about it:
            let mut failure_occurences = HashMap::new();
            for element in sorted_strings {
                let existing_value = failure_occurences.entry(element).or_insert(0);
                *existing_value += 1;
            }
            debug!("Failure occurences: {:#?}", failure_occurences);
            let worth_notifying = failure_occurences
                .iter()
                .filter(|&(_k, v)| *v == 3)
                .map(|(k, _v)| k.to_string())
                .collect::<String>();

            if worth_notifying.is_empty() {
                (ok_message, true)
            } else {
                (worth_notifying, false)
            }
        };
        let last_notifications_file = "/tmp/krecik-last-notification";
        let last_notifications =
            utilities::read_text_file(&last_notifications_file).unwrap_or_default();
        debug!(
            "Last notifications: {:?} == {:?}",
            notification_contents.0, last_notifications,
        );
        if notification_contents.0.is_empty() {
            debug!("No notification required.");
        } else if last_notifications == notification_contents.0 {
            debug!("Repeated notification skipped.");
        } else {
            fs::remove_file(&last_notifications_file).unwrap_or_default();
            utilities::write_append(&last_notifications_file, &notification_contents.0);
            warn!(
                "Sending notification, type: {}, with message: {}",
                if notification_contents.1 {
                    "SUCCESS"
                } else {
                    "FAILURE"
                },
                format!("{}", notification_contents.0)
            );
            // TODO: retry failed notifications (rare but happens) by additional error handling
            // TODO: read defined notifier webhook from configuration file
            if notification_contents.1 {
                utilities::notify_success("webhook", &notification_contents.0);
            } else {
                utilities::notify_failure("webhook", &notification_contents.0);
            }
        }
    }
}


impl Actor for Notificator {
    type Context = SyncContext<Self>;
}
