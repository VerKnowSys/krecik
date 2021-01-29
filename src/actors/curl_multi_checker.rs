use super::curl_generic_checker::GenericCurlChecker;
use crate::{checks::check::*, configuration::CHECK_TIMEOUT, products::story::*};
use actix::prelude::*;
use curl::multi::Multi;
use rayon::prelude::*;
use std::time::Duration;


/// CurlMultiChecker actor for Curl Multi bulk checks
#[derive(Debug, Copy, Clone)]
pub struct CurlMultiChecker;


/// List of Check(s)
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<Stories, Stories>")]
pub struct Checks(pub Vec<Check>);


impl Handler<Checks> for CurlMultiChecker {
    type Result = Result<Stories, Stories>;

    fn handle(&mut self, msg: Checks, _ctx: &mut Self::Context) -> Self::Result {
        let stories_from_domains = msg
            .clone()
            .0
            .into_par_iter()
            .flat_map(|check| {
                let notifier = check.notifier;
                check
                    .domains
                    .par_iter()
                    .flat_map(|domains| {
                        domains
                            .par_iter()
                            .flat_map(|domain| {
                                domain
                                    .expects
                                    .par_iter()
                                    .map(|expectation| {
                                        Self::check_ssl_expire(
                                            &domain.name,
                                            *expectation,
                                            notifier.clone(),
                                        )
                                    })
                                    .collect::<Stories>()
                            })
                            .collect::<Stories>()
                    })
                    .collect::<Stories>()
            })
            .collect::<Stories>();

        let stories_from_pages = msg
            .0
            .iter()
            .flat_map(|check| {
                let notifier = check.clone().notifier;
                check.pages.iter().flat_map(move |pages| {
                    let mut multi = Multi::new();
                    multi.pipelining(false, true).unwrap_or_default(); // disable http1.1, enable http2-multiplex

                    // collect tuple of page-checks and Curl handler:
                    let process_handlers: Vec<_> = pages
                        .iter()
                        .map(|check| (check, Self::load_handler_for(&check, &multi)))
                        .collect();

                    // perform all checks at once:
                    while multi.perform().unwrap_or_default() > 0 {
                        multi
                            .wait(&mut [], Duration::from_secs(CHECK_TIMEOUT))
                            .unwrap_or_default();
                    }

                    // Collect History of results:
                    process_handlers
                        .into_iter()
                        .flat_map(|(page, handler)| {
                            Self::process_page_handler(
                                &page,
                                handler,
                                &multi,
                                notifier.clone(),
                            )
                        })
                        .collect::<Stories>()
                })
            })
            .collect::<Stories>();
        Ok([stories_from_domains, stories_from_pages].concat())
    }
}


impl Actor for CurlMultiChecker {
    type Context = SyncContext<Self>;
}


impl GenericCurlChecker for CurlMultiChecker {}
