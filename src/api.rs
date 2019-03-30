use mime::*;
use colored::Colorize;

use crate::configuration::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::utilities::*;
use crate::checks::*;
use crate::checks::generic::*;
use crate::checks::pongo::*;
use crate::products::story::*;
use crate::products::history::*;


/**
    Public library API for Krecik remote-checks functionality
 **/


/// Execute single check by exact file
pub fn execute_checks_from_file(check_path: &str) -> History {
    debug!("Loading single check from file under path: {}", &check_path.cyan());
    GenCheck::load(&check_path)
        .and_then(|check| {
            debug!("Executing check: {}", format!("{:?}", check).magenta());
            Ok(check.execute())
        })
        .unwrap_or_else(|err| {
            let error = format!("Failed to load check from file: {}. Error details: {}", &check_path, err);
            error!("{}", error.red());
            History::new(Story::error(Unexpected::CheckParseProblem(error)))
        })
}


/// Execute all file checks from path
pub fn execute_checks_from_path(check_path: &str) -> History {
    debug!("Loading all checks from local path: {}/*.json", &check_path.cyan());
    History::new_from(
        list_check_files_from(&check_path)
            .into_iter()
            .flat_map(|check_resource| {
                let check = format!("{}/{}", check_path, check_resource);
                GenCheck::load(&check)
                    .and_then(|check| {
                        let debug = format!("{:?}", check);
                        debug!("Executing check: {}", debug.magenta());
                        Ok(check.execute())
                    })
                    .unwrap_or_else(|err| {
                        let error = format!("Failed to load check from file: {}. Error details: {}", &check, err);
                        error!("{}", error.red());
                        History::new(Story::error(Unexpected::CheckParseProblem(error)))
                    })
                    .stories()
            })
            .collect()
    )
}


/// Remote PongoHost check request
pub fn execute_checks_from_remote_resource_defined_in_path(check_path: &str) -> History {
    debug!("Loading checks from remote resources defined under path: {}/*.json", &check_path.cyan());
    History::new_from(
        list_check_files_from(&check_path)
            .into_iter()
            .flat_map(|check_file| {
                let check = format!("{}/{}", check_path, check_file);
                PongoHost::load(&check)
                    .and_then(|check| {
                        let debug = format!("Executing remote check: {:?}", check);
                        debug!("{}", debug.magenta());
                        Ok(check.execute())
                    })
                    .unwrap_or_else(|err| {
                        let error = format!("Failed to load remote check from file: {}. Error details: {}", &check, err);
                        error!("{}", error.red());
                        History::new(Story::error(Unexpected::CheckParseProblem(error)))
                    })
                    .stories()
            })
            .collect()
    )
}
