use hyper::*;
use mime::*;
use futures::{future, Future, Stream};
use gotham::helpers::http::response::create_response;
use gotham::state::{FromState, State};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::router::Router;


use crate::configuration::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::utilities::*;
use crate::inputs::check::*;
use crate::inputs::file::*;
use crate::products::history::*;


/// Execute all checks
pub fn handler_check_execute_by_name(state: State) -> (State, History) {
    let uri = Uri::borrow_from(&state).to_string();
    let name = uri.replace(CHECK_API_EXECUTE_REQUEST_PATH, "");
    let check_path = format!("tests/{}", &name);
    info!("Loading check from path: {}", &check_path);
    let history = FileCheck::load(&check_path)
        .and_then(|check| {
            Ok(check.execute())
        })
        .unwrap_or_else(|_| {
            error!("Failed to load check from file: '{}.json'", check_path);
            History::empty()
        });
    (state, history)
}


/// Web router:
pub fn router() -> Router {
    // use gotham::handler::assets::*;
    use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};


    build_simple_router(|route| {
        route
            .associate(
                &format!("{}:name", CHECK_API_EXECUTE_REQUEST_PATH), |handler| {
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

