use crate::{
    api::*,
    checks::{
        page::Method,
        pongo::{PongoCheck, PongoChecks},
    },
    configuration::{CHECKS_DIR, CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE},
    products::{
        expected::{Expected, PageExpectation, PageExpectations},
        unexpected::{Unexpected, UnexpectedMinor},
    },
};
use crate::{checks::generic::*, configuration::CHECK_TIMEOUT};
use crate::{
    checks::{check::*, page::Page},
    configuration::DEFAULT_SLACK_NAME,
};
use crate::{configuration::CHECK_CONNECTION_TIMEOUT, products::story::*};
use crate::{
    configuration::{CHECK_MAX_CONNECTIONS, CHECK_MAX_REDIRECTIONS},
    Collector,
};

// use crate::*;
use actix::prelude::*;
use chrono::*;
use colored::Colorize;
use fern::InitError;
use log::*;
use std::fs;
use std::{env, env::var, path::Path};

// use curl::easy::{Handler, WriteError};
use curl::{
    easy::{Easy2, List},
    multi::{Easy2Handle, Multi},
    Error as CurlError, MultiError,
};
use rayon::prelude::*;
use ssl_expiration2::SslExpiration;
use std::{
    io::{Error, ErrorKind},
    time::Duration,
};

use super::curl_generic_checker::GenericCurlChecker;


/// CurlMultiCheckerPongo actor for Curl Multi bulk checks
#[derive(Debug, Copy, Clone)]
pub struct CurlMultiCheckerPongo;


/// Actor message wrapper structure
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<Stories, Stories>")]
pub struct Checks(pub Vec<Check>);


impl Handler<Checks> for CurlMultiCheckerPongo {
    type Result = Result<Stories, Stories>;

    fn handle(&mut self, msg: Checks, _ctx: &mut Self::Context) -> Self::Result {
        Ok(msg
            .0
            .iter()
            .flat_map(|check| {
                check.pages.iter().flat_map(move |pages| {
                    let mut multi = Multi::new();
                    debug!("Starting new check: {:#?}", check);
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
                        .map(|(check, handler)| {
                            Self::process_page_handler(&check, handler, &multi)
                        })
                        .flat_map(|e| e.clone())
                        .collect::<Stories>()
                })
            })
            .collect())
    }
}


impl Actor for CurlMultiCheckerPongo {
    type Context = SyncContext<Self>;
}


impl GenericCurlChecker for CurlMultiCheckerPongo {}