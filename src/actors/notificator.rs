use crate::{products::story::*, utilities};
use actix::prelude::*;
use colored::Colorize;
use std::fs;


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
        let notification_contents = if stories.0.is_empty() {
            ("All services are UP again!".to_string(), true)
        } else {
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
            utilities::remove_duplicates(&mut sorted_strings);
            if sorted_strings.len() > 1 {
                debug!("Detected distinct errors. No notification to avoid spam.");
                (String::new(), false)
            } else {
                (sorted_strings.join(""), false)
            }
        };
        let last_notifications_file = "/tmp/krecik-last-failures";
        let last_notifications =
            utilities::read_text_file(&last_notifications_file).unwrap_or_default();
        debug!(
            "Last notifications: {:?} == {:?}",
            notification_contents.0, last_notifications,
        );
        if last_notifications == notification_contents.0 {
            info!("Notification already sent! Skipping.");
        } else if notification_contents.0.is_empty() {
            info!("Notification skipped since there are more than one failure detected.");
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
                format!("{}", notification_contents.0.yellow())
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
