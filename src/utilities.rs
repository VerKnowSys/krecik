use glob::glob;
use slack_hook::{AttachmentBuilder, PayloadBuilder, Slack};
use std::{
    fs::{self, OpenOptions},
    io::{prelude::*, Error},
    path::Path,
};

use crate::*;


/// Read single Check from text file, return error on parse error
pub fn read_single_check_result(check_path: &str) -> Result<Check, Error> {
    read_text_file(&check_path).and_then(|file_contents| {
        serde_json::from_str(&*file_contents)
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
    })
}


/// Read single Check from text file
pub fn read_single_check(check_path: &str) -> Option<Check> {
    let result = read_text_file(&check_path).and_then(|file_contents| {
        serde_json::from_str(&*file_contents)
            .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
    });
    match result {
        Ok(check) => Some(check),
        Err(err) => {
            debug!(
                "Error reading Check from path: {}. Cause: {}",
                check_path, err
            );
            None
        }
    }
}


/// Warns about notifiers undefined in dynamic configuration:
pub fn warn_for_undefined_notifiers(stories: &[Story]) {
    let notifiers = Config::load().notifiers.unwrap_or_default();
    let notifier_names = notifiers
        .into_iter()
        .map(|notifier| notifier.name)
        .collect::<Vec<_>>();
    let mut undefined = stories
        .iter()
        .cloned()
        .filter(|elem| !notifier_names.contains(&elem.notifier.clone().unwrap_or_default()))
        .filter_map(|elem| elem.notifier)
        .collect::<Vec<String>>();
    undefined.dedup();
    undefined.iter().for_each(|notifier| {
        warn!(
            "Notifier: '{}' is not defined in configuration file. Notifications won't be sent!",
            &notifier
        )
    });
}


/// Sends generic notification over Slack
pub fn notify(webhook: &str, message: &str, icon: &str, fail: bool) {
    Slack::new(webhook)
        .and_then(|slack| {
            PayloadBuilder::new()
                .username(DEFAULT_SLACK_NAME)
                .icon_emoji(icon)
                .attachments(vec![
                    if fail {
                        AttachmentBuilder::new(message)
                            .color(DEFAULT_SLACK_FAILURE_COLOR)
                            .build()
                            .unwrap_or_default()
                    } else {
                        AttachmentBuilder::new(message)
                            .color(DEFAULT_SLACK_SUCCESS_COLOR)
                            .build()
                            .unwrap_or_default()
                    },
                ])
                .build()
                .and_then(|payload| {
                    debug!("Sending notification with payload: {:?}", &payload);
                    slack.send(&payload)
                })
        })
        .unwrap_or_default(); // just ignore case when notification throws an error
}


/// Sends success notification to Slack
pub fn notify_success(webhook: &str, message: &str) {
    let success_emoji = Config::load()
        .success_emoji
        .unwrap_or_else(|| String::from(DEFAULT_SLACK_SUCCESS_ICON));
    notify(webhook, message, &success_emoji, false)
}


/// Sends failure notification to Slack
pub fn notify_failure(webhook: &str, message: &str) {
    let failure_emoji = Config::load()
        .failure_emoji
        .unwrap_or_else(|| String::from(DEFAULT_SLACK_FAILURE_ICON));
    notify(webhook, message, &failure_emoji, true)
}


/// Produce list of absolute paths to all files matching given glob pattern:
pub fn produce_list_absolute(glob_pattern: &str) -> Vec<String> {
    let mut list = vec![];
    for entry in glob(&glob_pattern).unwrap() {
        match entry {
            Ok(path) => {
                if let Some(element) = path.to_str() {
                    list.push(element.to_string())
                }
            }
            Err(err) => {
                error!("Error: produce_list(): {}", err);
            }
        }
    }
    trace!("produce_list_absolute(): Elements: {:?}", list);
    list
}


/// List all check files from given dir, also considering krecik_root value
pub fn list_all_checks_from(checks_dir: &str) -> Vec<String> {
    let krecik_root_dir = Config::load().krecik_root.unwrap_or_default();
    let glob_pattern = if !Path::new(&krecik_root_dir).exists() {
        if !krecik_root_dir.is_empty() {
            warn!(
                "Krecik root directory doesn't exists: {}. Falling back to current directory.",
                krecik_root_dir
            );
        } else {
            warn!("Krecik root directory wasn't specified, using current directory.");
        }
        format!("{}/**/*.json", checks_dir)
    } else {
        format!("{}/{}/**/*.json", krecik_root_dir, checks_dir)
    };
    debug!("list_all_checks_from(): {}", glob_pattern);
    produce_list_absolute(&glob_pattern)
}


/// Read text file
pub fn read_text_file(name: &str) -> Result<String, Error> {
    fs::read_to_string(name)
}


/// Write-once-and-atomic to a file
pub fn write_append(file_path: &str, contents: &str) {
    // NOTE: since file is written in "write only, all at once" mode, we have to be sure not to write empty buffer
    if !contents.is_empty() {
        let mut options = OpenOptions::new();
        match options.create(true).append(true).open(&file_path) {
            Ok(mut file) => {
                file.write_all(contents.as_bytes()).unwrap_or_else(|_| {
                    panic!("Access denied? File can't be written: {}", &file_path)
                });
                debug!("Atomically written data to file: {}", &file_path);
            }

            Err(err) => {
                error!(
                    "Atomic write to: {} has failed! Cause: {}",
                    &file_path,
                    err.to_string()
                )
            }
        }
    }
}


/// Extracts file name from full path
pub fn file_name_from_path(path: &str) -> String {
    let path = Path::new(path);
    path.file_name()
        .unwrap_or_default()
        .to_os_string()
        .into_string()
        .unwrap_or_default()
}
