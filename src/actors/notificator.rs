use crate::{
    debug, info, products::story::*, trace, utilities, Config,
    CHECK_DEFAULT_SUCCESS_NOTIFICATION_MSG,
};
use actix::prelude::*;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};


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
        trace!("Defined notifiers: {notifiers:#?}");

        let ok_message = Config::load()
            .ok_message
            .unwrap_or_else(|| String::from(CHECK_DEFAULT_SUCCESS_NOTIFICATION_MSG));

        for a_notifier in &notifiers {
            let notifier_name = a_notifier.name.to_owned();
            let mut sorted_errors = stories
                .0
                .iter()
                .filter_map(|story| {
                    if notifier_name == story.notifier.to_owned().unwrap_or_default() {
                        story.error.to_owned().map(|error| error.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            // sorted_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted_errors.sort();

            // let's iterate over each string and count occurrences
            // if there are 3 occurrences - we should send notification about it:
            let mut failure_occurrences = HashMap::new();
            for element in sorted_errors {
                let existing_value = failure_occurrences.entry(element).or_insert(0);
                *existing_value += 1;
            }
            if !failure_occurrences.is_empty() {
                info!(
                    "Notifier: {}, failure occurrences: {:#?}",
                    notifier_name, failure_occurrences
                );
            }

            let errors_with_webhooks = failure_occurrences
                .iter()
                .filter_map(|(error, value)| {
                    if value >= &3 {
                        Some((format!("{error}\n"), a_notifier.slack_webhook.to_owned()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

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
                    "History of failures for notifier: {notifier_name}: {history_of_failures:?}"
                );
                if history_of_failures.is_empty() {
                    debug!("No need to send notification to notificator: {notifier_name}");
                } else {
                    info!(
                        "Sending SUCCESS notification for notifier: {notifier_name}, with message: {ok_message}"
                    );
                    utilities::notify_success(&a_notifier.slack_webhook, &ok_message); // TODO: Since Slack API can fail… retry crate could be used
                    drop(history); // drop mutex lock
                    let mut history = NOTIFY_HISTORY.lock().unwrap();
                    history.retain(|(_, _, notifier, _)| notifier != &notifier_name);
                }
            } else {
                for (message, webhook) in &errors_with_webhooks {
                    let notified_entry = (
                        false,
                        message.to_owned(),
                        notifier_name.to_owned(),
                        webhook.to_owned(),
                    );
                    let unnotified_entry = (
                        true,
                        message.to_owned(),
                        notifier_name.to_owned(),
                        webhook.to_owned(),
                    );
                    let mut history = NOTIFY_HISTORY.lock().unwrap();
                    if history.contains(&notified_entry) {
                        debug!("Already notified message skipped: '{message}'");
                    } else {
                        debug!("Pushing new entry: {unnotified_entry:?}");
                        history.push(unnotified_entry)
                    }
                }
            }
        }

        // iterate again over notifiers, determine webhooks and group messages together to send failure notification
        for a_notifier in notifiers {
            let notifier_name = a_notifier.name;
            let history = NOTIFY_HISTORY.lock().unwrap();
            let filter = history.iter().filter(|(to_notify, _, notifier, _)| {
                notifier == &notifier_name && *to_notify
            });
            let failure_messages = filter
                .to_owned()
                .map(|(_, message, ..)| message.to_string())
                .collect::<Vec<_>>();
            let webhook = filter // webhook is always one per notifier
                .map(|(_, _, _, webhook)| webhook.to_string())
                .take(1)
                .collect::<String>();

            if failure_messages.is_empty() {
                debug!("No new failure messages for notifier {notifier_name}!");
            } else {
                let messages = failure_messages.join("");
                info!(
                    "Sending failure notification: '{messages}' to notifier id: {notifier_name}"
                );
                trace!("webhook: {webhook}");
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
        debug!("NOTIFY_HISTORY state: {history:?}");
    }
}


impl Actor for Notificator {
    type Context = SyncContext<Self>;
}
