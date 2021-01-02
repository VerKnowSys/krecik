use crate::{checks::pongo::*, *};
use colored::Colorize;
use rayon::prelude::*;
use std::io::{Error, ErrorKind};


/**
   Public library API for Krecik remote-checks functionality
**/

/// Return checks from path, excluding remotes
pub fn all_checks_but_remotes() -> Vec<Check> {
    list_all_checks_from(CHECKS_DIR)
        .par_iter()
        .filter_map(|check_path| {
            if !check_path.contains(REMOTE_CHECKS_DIR) && !check_path.contains(TESTS_DIR) {
                read_text_file(&check_path)
                    .and_then(|file_contents| {
                        serde_json::from_str(&*file_contents).map_err(|err| {
                            Error::new(ErrorKind::InvalidInput, err.to_string())
                        })
                    })
                    .unwrap_or_default()
            } else {
                None
            }
        })
        .collect()
}


/// Return remote domain+pages checks via mapper
pub fn all_checks_pongo_merged() -> Vec<Check> {
    list_all_checks_from(&format!("{}/{}", CHECKS_DIR, REMOTE_CHECKS_DIR))
        .into_iter()
        .map(|pongo_mapper| {
            let mapper = read_pongo_mapper(&pongo_mapper);
            let domain_checks = get_pongo_hosts(&mapper.url)
                .into_par_iter()
                .flat_map(|check| collect_pongo_domains(&check))
                .collect();
            let pongo_checks = get_pongo_hosts(&mapper.url)
                .into_par_iter()
                .flat_map(|check| collect_pongo_hosts(&check, &mapper))
                .collect();

            Check {
                pages: Some(pongo_checks),
                domains: Some(domain_checks),

                // pass alert webhook and channel from mapper to the checks
                alert_webhook: mapper.alert_webhook,
                alert_channel: mapper.alert_channel,
                ..Check::default()
            }
        })
        .collect()
}


/// Return remote domain checks via mapper
pub fn all_checks_pongo_remote_domains() -> Vec<Check> {
    list_all_checks_from(&format!("{}/{}", CHECKS_DIR, REMOTE_CHECKS_DIR))
        .into_par_iter()
        .map(get_domain_checks)
        .collect()
}


/// Return remote page checks via mapper
pub fn all_checks_pongo_remote_pages() -> Vec<Check> {
    list_all_checks_from(&format!("{}/{}", CHECKS_DIR, REMOTE_CHECKS_DIR))
        .into_par_iter()
        .map(get_page_checks)
        .collect()
}


/// Execute single check by exact file
#[deprecated(since = "0.9.0")]
pub fn execute_checks_from_file(check_path: &str) -> History {
    debug!(
        "Loading single check from file under path: {}",
        &check_path.cyan()
    );
    GenCheck::load(&check_path)
        .map(|check| {
            let file_name = file_name_from_path(check_path);
            debug!("Executing check: {}", file_name.magenta());
            check.execute(&file_name)
        })
        .unwrap_or_else(|err| {
            let error = format!(
                "Failed to load check from file: {}. Error details: {}",
                &check_path, err
            );
            error!("{}", error.red());
            History::new(Story::error(Unexpected::CheckParseProblem(error)))
        })
}


/// Execute all file checks from path
#[deprecated(since = "0.9.0")]
pub fn execute_checks_from_path(check_path: &str) -> History {
    debug!(
        "Loading all checks from local path: {}/*.json",
        &check_path.cyan()
    );
    History::new_from(
        list_check_files_from(&check_path)
            .into_iter()
            .flat_map(|check_resource| {
                let check_file = format!("{}/{}", check_path, check_resource);
                GenCheck::load(&check_file)
                    .map(|check| {
                        let file_name = file_name_from_path(&check_file);
                        debug!("Executing check from file: {}", file_name.magenta());
                        check.execute(&file_name)
                    })
                    .unwrap_or_else(|err| {
                        let error = format!(
                            "Failed to load check from file: {}. Error details: {}",
                            &check_file, err
                        );
                        error!("{}", error.red());
                        History::new(Story::error(Unexpected::CheckParseProblem(error)))
                    })
                    .stories()
            })
            .collect(),
    )
}


/// Remote PongoCheck check request
#[deprecated(since = "0.9.0")]
pub fn execute_checks_from_remote_resource_defined_in_path(check_path: &str) -> History {
    debug!(
        "Loading checks from remote resources defined under path: {}",
        &check_path.cyan()
    );
    History::new_from(
        list_check_files_from(&check_path)
            .into_iter()
            .flat_map(|check_file| {
                let mapper = format!("{}/{}", check_path, check_file);
                debug!("Mapper file: {}", mapper);
                PongoCheck::load(&mapper)
                    .map(|check| {
                        let file_name = file_name_from_path(&check_file);
                        debug!("Executing remote check from file: {}", file_name);
                        check.execute(&file_name)
                    })
                    .unwrap_or_else(|err| {
                        let error = format!(
                            "Failed to load remote check from file: {}. Error details: {}",
                            &mapper, err
                        );
                        error!("{}", error.red());
                        History::new(Story::error(Unexpected::CheckParseProblem(error)))
                    })
                    .stories()
            })
            .collect(),
    )
}
