use crate::{products::story::*, utilities, Config};
use actix::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;


lazy_static! {
    /// List of (to_notify, message, notifier name, webhook) tuples:
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
            let notifier_name = a_notifier.clone().name;
            let mut sorted_errors = stories
                .0
                .iter()
                .filter(|elem| notifier_name == elem.notifier.clone().unwrap_or_default())
                .map(|elem| elem.error.clone().unwrap().to_string())
                .collect::<Vec<String>>();
            sorted_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // let's iterate over each string and count occurences
            // if there are 3 occurences - we should send notification about it:
            let mut failure_occurences = HashMap::new();
            for element in sorted_errors {
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
                .map(|(error, _v)| (format!("{}\n", error), a_notifier.clone().slack_webhook))
                .collect::<Vec<(String, String)>>();

            // no errors, means that we can traverse NOTIFY_HISTORY and pick all previously failed entries and send ok_message
            if errors_with_webhooks.is_empty() {
                let history = NOTIFY_HISTORY.lock().unwrap();
                let history_of_failures = history
                    .iter()
                    .filter(|(to_notify, _, notifier, _)| {
                        notifier == &notifier_name && !*to_notify
                    })
                    .collect::<Vec<_>>();

                debug!(
                    "History of failures for notifier: {}: {:?}",
                    &notifier_name, history_of_failures
                );
                if history_of_failures.is_empty() {
                    debug!(
                        "No need to send notification to notificator: {}",
                        &notifier_name
                    );
                } else {
                    info!(
                        "Sending SUCCESS notification for notifier: {}, with message: {}",
                        &notifier_name, &ok_message
                    );
                    let webhook = a_notifier.slack_webhook; // TODO: add notification mechanisms other than Slack?
                    utilities::notify_success(&webhook, &ok_message); // TODO: Since Slack API can fail… retry crate could be used
                    drop(history); // drop mutex lock
                    let mut history = NOTIFY_HISTORY.lock().unwrap();
                    history.retain(|(_, _, notifier, _)| notifier != &notifier_name);
                }
            } else {
                for (message, webhook) in errors_with_webhooks.clone() {
                    let notified_entry = (
                        false,
                        message.clone(),
                        notifier_name.clone(),
                        webhook.clone(),
                    );
                    let unnotified_entry = (
                        true,
                        message.clone(),
                        notifier_name.clone(),
                        webhook.clone(),
                    );
                    let mut history = NOTIFY_HISTORY.lock().unwrap();
                    if history.contains(&notified_entry) {
                        debug!("Already notified message skipped: {}", &message);
                    } else {
                        debug!("Pushing new entry: {:?}", unnotified_entry);
                        history.push(unnotified_entry)
                    }
                }
            }
        }

        // iterate again over notifiers, determine webhooks and group messages together to send failure notification
        for a_notifier in notifiers.clone() {
            let notifier_name = a_notifier.name;
            let history = NOTIFY_HISTORY.lock().unwrap();
            let failure_messages = history
                .iter()
                .filter(|(to_notify, _, notifier, _)| notifier == &notifier_name && *to_notify)
                .map(|(_, message, ..)| message.to_string())
                .collect::<Vec<_>>();
            let webhook = history // webhook is same for same notifier
                .iter()
                .filter(|(to_notify, _, notifier, _)| notifier == &notifier_name && *to_notify)
                .map(|(_, _, _, webhook)| webhook.to_string())
                .take(1)
                .collect::<String>();

            if failure_messages.is_empty() {
                debug!(
                    "Failure messages already notfied: {}",
                    &failure_messages.join("")
                );
            } else {
                let messages = failure_messages.join("");
                info!(
                    "Sending FAILURE notification: '{}' to notifier id: {}, webhook: '{}'",
                    &messages, &notifier_name, &webhook
                );
                utilities::notify_failure(&webhook, &messages);

                drop(history);
                let mut history = NOTIFY_HISTORY.lock().unwrap();
                history
                    .iter_mut()
                    .filter(|(to_notify, _, notifier, _)| {
                        notifier == &notifier_name && *to_notify
                    })
                    .for_each(|(to_notify, ..)| *to_notify = false);
            }
        }

        let history = NOTIFY_HISTORY.lock().unwrap();
        debug!("NOTIFY_HISTORY state: {:?}", history);
    }
}


impl Actor for Notificator {
    type Context = SyncContext<Self>;
}
