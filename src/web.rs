use gotham::router::Router;
use gotham::state::State;

use crate::configuration::*;
use crate::products::expected::*;
use crate::products::unexpected::*;
use crate::utilities::*;
use crate::inputs::check::*;
use crate::inputs::file::*;


/// Execute all checks
pub fn handler_check_execute_all(state: State) -> (State, History) {
    let story = Story::new(
        3,
        Unexpected::FailedDomain("domain.tld".to_string(), DomainExpectation::ValidResolvable)
    );
    let history = History::new(
        Story::new(
            1,
            Unexpected::FailedDomain("domain.tld".to_string(), DomainExpectation::ValidResolvable)
        )
    );

    FileCheck::load("tests/test1")
        .and_then(|check| {
            Ok(check.execute().unwrap())
        })
        .unwrap_or_default();

    (state, history.append(story))
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

