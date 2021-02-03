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
            "All services are UP again!".to_string()
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
            sorted_strings.join("")
        };
        let last_notifications_file = "/tmp/krecik-last-failures";
        let last_notifications =
            utilities::read_text_file(&last_notifications_file).unwrap_or_default();
        debug!(
            "Last notifications: {:?} == {:?}",
            notification_contents, last_notifications,
        );
        if last_notifications == notification_contents {
            debug!("Notification already sent! Skipping.");
        } else {
            fs::remove_file(&last_notifications_file).unwrap_or_default();
            utilities::write_append(&last_notifications_file, &notification_contents);
            warn!(
                "SEND NOTIFICATION STORIES: {}",
                format!("{}", notification_contents.yellow())
            );
        }
    }
}


impl Actor for Notificator {
    type Context = SyncContext<Self>;
}
