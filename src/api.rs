use crate::{checks::pongo::*, *};
use rayon::prelude::*;
use std::io::{Error, ErrorKind};


/**
   Public library API for Krecik remote-checks functionality
**/

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


/// Return checks from path, excluding remotes
pub fn all_checks_but_remotes() -> Vec<Check> {
    list_all_checks_from(CHECKS_DIR)
        .par_iter()
        .filter_map(|check_path| {
            if !check_path.contains(REMOTE_CHECKS_DIR) && !check_path.contains(TESTS_DIR) {
                // select only valid Check, just ignore any malformed ones
                read_single_check(&check_path)
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
            let all_pongo_checks = get_pongo_checks(&mapper.url);
            let domain_checks = all_pongo_checks
                .clone()
                .into_par_iter()
                .flat_map(|check| collect_pongo_domains(&check))
                .collect();
            let pongo_checks = all_pongo_checks
                .into_par_iter()
                .flat_map(|check| collect_pongo_hosts(&check, &mapper))
                .collect();

            Check {
                pages: Some(pongo_checks),
                domains: Some(domain_checks),
                notifier: mapper.notifier,
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
