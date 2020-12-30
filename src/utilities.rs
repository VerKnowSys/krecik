use glob::glob;
use slack_hook::{AttachmentBuilder, PayloadBuilder, Slack};
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;

use crate::*;


/// Sends generic notification over Slack
pub fn notify(webhook: &str, channel: &str, message: &str, icon: &str, fail: bool) {
    Slack::new(webhook)
        .and_then(|slack| {
            PayloadBuilder::new()
                .channel(channel)
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
pub fn notify_success(webhook: &str, channel: &str, message: &str) {
    notify(webhook, channel, message, DEFAULT_SLACK_SUCCESS_ICON, false)
}


/// Sends failure notification to Slack
pub fn notify_failure(webhook: &str, channel: &str, message: &str) {
    notify(webhook, channel, message, DEFAULT_SLACK_FAILURE_ICON, true)
}


/// Produce list of absolute paths to all files matching given glob pattern:
pub fn produce_list_absolute(glob_pattern: &str) -> Vec<String> {
    let mut list = vec![];
    for entry in glob(&glob_pattern).unwrap() {
        match entry {
            Ok(path) => {
                match path.to_str() {
                    Some(element) => list.push(element.to_string()),
                    None => (),
                }
            }
            Err(err) => {
                error!("Error: produce_list(): {}", err);
            }
        }
    }
    debug!("produce_list_absolute(): Elements: {:?}", list);
    list
}


/// Produce list of dirs/files matching given glob pattern:
pub fn produce_list(glob_pattern: &str) -> Vec<String> {
    let mut list = vec![];
    for entry in glob(&glob_pattern).unwrap() {
        match entry {
            Ok(path) => {
                match path.file_name().unwrap_or_default().to_str() {
                    Some(element) => list.push(element.to_string()),
                    None => (),
                }
            }
            Err(err) => {
                error!("Error: produce_list(): {}", err);
            }
        }
    }
    debug!("produce_list(): Elements: {:?}", list);
    list
}


/// List all check files found in default checks dir
pub fn list_check_files() -> Vec<String> {
    list_check_files_from(CHECKS_DIR)
}


/// List all check files from given dir
pub fn list_check_files_from(checks_dir: &str) -> Vec<String> {
    let glob_pattern = format!("{}/*.json", checks_dir);
    debug!("list_check_files(): {}", glob_pattern);
    produce_list(&glob_pattern)
}


/// List all check files from given dir
pub fn list_all_checks_from(checks_dir: &str) -> Vec<String> {
    let glob_pattern = format!("{}/**/*.json", checks_dir);
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
