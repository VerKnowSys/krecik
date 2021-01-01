use crate::checks::pongo::*;
use crate::*;
use colored::Colorize;
use curl::easy::Easy2;
use rayon::prelude::*;
use regex::Regex;
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


/// Return remote domain checks via mapper
pub fn all_checks_pongo_remote_domains() -> Vec<Check> {
    list_all_checks_from(&format!("{}/{}", CHECKS_DIR, REMOTE_CHECKS_DIR))
        .into_par_iter()
        .map(|pongo_mapper| {
            let mapper: PongoRemoteMapper = read_text_file(&pongo_mapper)
                .and_then(|file_contents| {
                    serde_json::from_str(&file_contents)
                        .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
                })
                .unwrap_or_default();

            let mut easy = Easy2::new(Collector(Vec::new()));
            easy.get(true).unwrap_or_default();
            easy.url(&mapper.url).unwrap_or_default();
            easy.perform().unwrap_or_default();
            // .expect(&format!("Expected something from remote: {}", pongo_mapper));
            let contents = easy.get_ref();
            let remote_raw = String::from_utf8_lossy(&contents.0);

            // now use default Pongo structure defined as default for PongoRemoteMapper
            let pongo_hosts: PongoChecks = serde_json::from_str(&remote_raw)
                .map_err(|err| error!("Failed to parse Pongo input: {:#?}", err))
                .unwrap_or_default();

            let domain_checks = pongo_hosts
                .into_par_iter()
                .flat_map(|host| {
                    host.data
                        .host
                        .unwrap_or_default()
                        .vhosts
                        .and_then(|vhosts| {
                            vhosts
                                .par_iter()
                                .filter(|vhost| !vhost.starts_with("*.")) // filter out wildcard domains
                                .map(|vhost| {
                                    Some(Domain {
                                        name: vhost.to_string(),
                                        expects: default_domain_expectations(),
                                    })
                                })
                                .collect::<Option<Domains>>()
                        })
                        .unwrap_or_default()
                })
                .collect();
            Check {
                domains: Some(domain_checks),

                // pass alert webhook and channel from mapper to the checks
                alert_webhook: mapper.alert_webhook,
                alert_channel: mapper.alert_channel,
                ..Check::default()
            }
        })
        .collect()
}


/// Return remote page checks via mapper
pub fn all_checks_pongo_remote_pages() -> Vec<Check> {
    list_all_checks_from(&format!("{}/{}", CHECKS_DIR, REMOTE_CHECKS_DIR))
        .into_par_iter()
        .map(|pongo_mapper| {
            let mapper: PongoRemoteMapper = read_text_file(&pongo_mapper)
                .and_then(|file_contents| {
                    serde_json::from_str(&file_contents)
                        .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))
                })
                .unwrap_or_default();

            let mut easy = Easy2::new(Collector(Vec::new()));
            easy.get(true).unwrap_or_default();
            easy.url(&mapper.url).unwrap_or_default();
            easy.perform().unwrap_or_default();
            // .expect(&format!("Expected something from remote: {}", pongo_mapper));
            let contents = easy.get_ref();
            let remote_raw = String::from_utf8_lossy(&contents.0);

            // now use default Pongo structure defined as default for PongoRemoteMapper
            let pongo_hosts: PongoChecks = serde_json::from_str(&remote_raw)
                .map_err(|err| error!("Failed to parse Pongo input: {:#?}", err))
                .unwrap_or_default();

            debug!("Pongo hosts: {:#?}", &pongo_hosts);
            let pongo_checks = pongo_hosts
                .clone()
                .into_par_iter()
                .flat_map(|host| {
                    let ams = host.clone().data.ams.unwrap_or_default();
                    let active = host.active.unwrap_or(false);
                    let client = host.clone().client.unwrap_or_default();
                    let options = host.clone().options;

                    let pongo_private_token = Regex::new(r"\?token=[A-Za-z0-9_-]*").unwrap();
                    let safe_url =
                        pongo_private_token.replace(&mapper.url, "[[token-masked]]");
                    [
                        // merge two lists for URLs: "vhosts" and "showrooms":
                        host.clone()
                            .data
                            .host
                            .unwrap_or_default()
                            .vhosts
                            .and_then(|vhosts| {
                                vhosts
                                    .par_iter()
                                    .filter(|vhost| {
                                        !vhost.starts_with("*.")
                                            && vhost.contains(
                                                &mapper
                                                    .only_vhost_contains
                                                    .clone()
                                                    .unwrap_or_default(),
                                            )
                                    }) // filter out wildcard domains and pick only these matching value of only_vhost_contains field
                                    .map(|vhost| {
                                        if active {
                                            Some(Page {
                                                url: format!(
                                                    "{}{}/{}/",
                                                    CHECK_DEFAULT_PROTOCOL, vhost, ams
                                                ),
                                                expects: pongo_page_expectations(),
                                                options: options.clone(),
                                            })
                                        } else {
                                            debug!("Skipping not active client: {}", &client);
                                            None
                                        }
                                    })
                                    .collect::<Option<Pages>>()
                            })
                            .unwrap_or_default(),
                        host.data
                            .host
                            .unwrap_or_default()
                            .showroom_urls
                            .and_then(|showrooms| {
                                showrooms
                                    .par_iter()
                                    .map(|vhost| {
                                        if active {
                                            Some(Page {
                                                url: vhost.to_string(),
                                                expects: showroom_page_expectations(),
                                                options: None,
                                            })
                                        } else {
                                            debug!("Skipping not active client: {}", &client);
                                            None
                                        }
                                    })
                                    .collect::<Option<Pages>>()
                            })
                            .unwrap_or_default(),
                    ]
                    .concat()
                })
                .collect();

            Check {
                pages: Some(pongo_checks),
                // domains: Some(domain_checks),

                // pass alert webhook and channel from mapper to the checks
                alert_webhook: mapper.alert_webhook,
                alert_channel: mapper.alert_channel,
                ..Check::default()
            }
        })
        .collect()
}


/// Execute single check by exact file
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
