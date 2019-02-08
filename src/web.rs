use gotham::router::Router;
use gotham::state::State;

use ssl_expiration::SslExpiration;
use curl::easy::{Easy2, Handler, WriteError};

use crate::configuration::*;
use crate::products::expected::*;
use crate::products::unexpected::*;


struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}


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


    let expiration = SslExpiration::from_domain_name("google.com").unwrap();
    if expiration.is_expired() {
        // do something if SSL certificate expired
        panic!("Oh expired domain. So shame. I will die now cuz nothing really matters ;)");
    }
    info!("Domain: {} - Total days before expiration: {}. Total seconds before expiration: {}", "google.com", expiration.days(), expiration.secs());


    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap();
    easy.verbose(true).unwrap();
    easy.url("https://www.rust-lang.org/").unwrap();
    easy.perform().unwrap();

    assert_eq!(easy.response_code().unwrap(), 200);
    let _contents = easy.get_ref();
    // println!("{}", String::from_utf8_lossy(&contents.0));


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

