use crate::{products::story::*, Config};
use actix::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;


lazy_static! {
    /// List of (is_failure, message, notifier name, webhook) tuples:
    static ref NOTIFY_HISTORY: Mutex<Vec<(bool, String, String, String)>> = Mutex::new({
        #[allow(unused_mut)]
        let mut history = Vec::new();
        history
    });
}


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

        let ok_message = Config::load()
            .ok_message
            .unwrap_or_else(|| String::from("All services are UP."));

        for a_notifier in notifiers.clone() {
            let notifier_name = a_notifier.name;
            let mut sorted_tuples = stories
                .0
                .iter()
                .filter(|elem| notifier_name == elem.notifier.clone().unwrap_or_default())
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
            debug!(
                "Notifier {} failure occurences: {:#?}",
                notifier_name, failure_occurences
            );

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
                    (format!("{}\n", tuple.clone().0), notifier)
                })
                .collect::<Vec<(String, String)>>();

            // no errors, means that we can traverse NOTIFY_HISTORY and pick all previously failed entries and send ok_message
            if errors_with_webhooks.is_empty() {
                let history = NOTIFY_HISTORY.lock().unwrap();
                let history_of_failures = history
                    .iter()
                    .filter(|(is_failure, _, notifier, _)| {
                        notifier == &notifier_name && *is_failure
                    })
                    .collect::<Vec<_>>();

                debug!(
                    "History of failures for notifier: {}: {:?}",
                    &notifier_name, history_of_failures
                );
                if history_of_failures.is_empty() {
                    info!(
                        "No need to send notification to notificator: {}",
                        &notifier_name
                    );
                } else {
                    warn!(
                        "OK notification for notifier: {}, with message: {}",
                        &notifier_name, &ok_message
                    );
                    drop(history); // drop mutex lock
                    let mut history = NOTIFY_HISTORY.lock().unwrap();
                    history.retain(|(is_failure, _, notifier, _)| {
                        notifier != &notifier_name && *is_failure
                    });
                }
            } else {
                for (message, webhook) in errors_with_webhooks.clone() {
                    let history = NOTIFY_HISTORY.lock().unwrap();
                    let last_notifications = history
                        .iter()
                        .filter(|(is_failure, _, notifier, _)| {
                            notifier == &notifier_name && *is_failure
                        })
                        .cloned()
                        .collect::<Vec<_>>();

                    if last_notifications.contains(&(
                        true,
                        message.clone(),
                        notifier_name.clone(),
                        webhook.clone(),
                    )) {
                        info!(
                            "Repeated FAIL notification skipped for for notifer: {} with message: {}",
                            &notifier_name, &message
                        );
                    } else {
                        drop(history); // remove previous lock on NOTIFY_HISTORY!
                        let mut history = NOTIFY_HISTORY.lock().unwrap();
                        history.push((
                            true,
                            message.clone(),
                            notifier_name.clone(),
                            webhook.clone(),
                        ));

                        warn!(
                            "Sending FAIL notification: '{}' to notifier id: {}, webhook: '{}'",
                            &message.clone(),
                            &notifier_name,
                            &webhook
                        );
                        // utilities::notify_failure(&webhook, &ok_message);
                    }
                }
            }
        }

        let history = NOTIFY_HISTORY.lock().unwrap();
        warn!("NOTIFY_HISTORY: {:?}", history);
    }
}


impl Actor for Notificator {
    type Context = SyncContext<Self>;
}
