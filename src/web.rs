use gotham::router::Router;
use gotham::state::State;

use crate::configuration::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::utilities::*;
use crate::inputs::check::*;
use crate::inputs::file::*;
use crate::products::history::*;


/// Execute all checks
pub fn handler_check_execute_all(state: State) -> (State, History) {
    let check = FileCheck::load("tests/test1").unwrap();
    let history = check.execute().unwrap();
    (state, history)
}


/// Web router:
pub fn router() -> Router {
    // use gotham::handler::assets::*;
    use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};


    build_simple_router(|route| {
        route
            .associate(
                &"/json/execute/all".to_string(), |handler| {
                    handler.get().to(handler_check_execute_all);
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

