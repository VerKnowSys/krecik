use hyper::*;
use mime::*;
use futures::{future, Future, Stream};
use gotham::helpers::http::response::create_response;
use gotham::state::{FromState, State};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::router::Router;
use colored::Colorize;

// use fern::colors::{Color, ColoredLevelConfig};


use crate::configuration::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::utilities::*;
use crate::inputs::check::*;
use crate::inputs::file::*;
use crate::products::history::*;


/// Execute single check by path/name
pub fn handler_check_execute_by_name(state: State) -> (State, History) {
    let uri = Uri::borrow_from(&state).to_string();
    let check_path = format!("{}/{}", &CHECKS_DIR, uri.replace(CHECK_API_EXECUTE_REQUEST_PATH, ""));
    info!("Loading single check from path: {}", &check_path.cyan());
    (state,
        FileCheck::load(&check_path)
            .and_then(|check| {
                let debug = format!("Executing check: {:#?}", check);
                debug!("{}", debug.magenta());
                Ok(check.execute())
            })
            .unwrap_or_else(|err| {
                let error = format!("Failed to load check from file: {}. Error details: {}", &check_path, err);
                error!("{}", error.red());
                History::new(Story::new_error(Some(Unexpected::CheckParseProblem(error))))
            })
    )
}


/// Execute all file checks from path
pub fn handler_check_execute_all_from_path(state: State) -> (State, History) {
    let uri = Uri::borrow_from(&state).to_string();
    let check_path = format!("{}{}", CHECKS_DIR, uri.replace(CHECK_API_EXECUTE_REQUEST_PATH, ""));
    info!("Loading all checks from path: {}", &check_path.cyan());
    (state,
        History::new_from(
            list_check_files_from(&check_path)
                .into_iter()
                .flat_map(|check_file| {
                    let check = format!("{}/{}", check_path, check_file);
                    FileCheck::load(&check)
                        .and_then(|check| {
                            let debug = format!("Executing check: {:#?}", check);
                            debug!("{}", debug.magenta());
                            Ok(check.execute())
                        })
                        .unwrap_or_else(|err| {
                            let error = format!("Failed to load check from file: {}. Error details: {}", &check, err);
                            error!("{}", error.red());
                            History::new(Story::new_error(Some(Unexpected::CheckParseProblem(error))))
                        })
                        .stories()
                })
                .collect()
        )
    )
}


/// Web router:
pub fn router() -> Router {
    // use gotham::handler::assets::*;
    use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};


    build_simple_router(|route| {
        route
            .associate(
                &format!("{}/:path", CHECK_API_EXECUTE_REQUEST_PATH), |handler| {
                    handler
                        .get()
                        .to(handler_check_execute_all_from_path);
                }
            );

        route
            .associate(
                &format!("{}/:path/:name", CHECK_API_EXECUTE_REQUEST_PATH), |handler| {
                    handler
                        .get()
                        .to(handler_check_execute_by_name);
                }
            );

        // route
        //     .get("/")
        //     .to_file(format!("{}/web/static/html/panel.html", PROJECT_DIRECTORY));

        // route
        //     .get("/*")
        //     .to_dir(FileOptions::new(format!("{}/web/", PROJECT_DIRECTORY))
        //     .with_gzip(true)
        //     .build()
        // );

        // route
        //     .get("/static/*")
        //     .to_dir(FileOptions::new(format!("{}/web/static/", PROJECT_DIRECTORY))
        //     .with_gzip(false)
        //     .build()
        // );

    })
}

