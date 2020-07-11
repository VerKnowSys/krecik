use crate::*;
use colored::Colorize;


/**
   Public library API for Krecik remote-checks functionality
**/

/// Execute single check by exact file
pub fn execute_checks_from_file(check_path: &str) -> History {
    debug!(
        "Loading single check from file under path: {}",
        &check_path.cyan()
    );
    GenCheck::load(&check_path)
        .and_then(|check| {
            let file_name = file_name_from_path(check_path);
            debug!("Executing check: {}", file_name.magenta());
            Ok(check.execute(&file_name))
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
                let check = format!("{}/{}", check_path, check_resource);
                GenCheck::load(&check)
                    .and_then(|check| {
                        let file_name = file_name_from_path(&check_path);
                        debug!("Executing check from file: {}", file_name.magenta());
                        Ok(check.execute(&file_name))
                    })
                    .unwrap_or_else(|err| {
                        let error = format!(
                            "Failed to load check from file: {}. Error details: {}",
                            &check, err
                        );
                        error!("{}", error.red());
                        History::new(Story::error(Unexpected::CheckParseProblem(error)))
                    })
                    .stories()
            })
            .collect(),
    )
}


/// Remote PongoHost check request
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
                PongoHost::load(&mapper)
                    .and_then(|check| {
                        let file_name = file_name_from_path(&check_file);
                        debug!("Executing remote check from file: {}", file_name);
                        Ok(check.execute(&file_name))
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
