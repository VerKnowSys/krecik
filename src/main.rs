//! "Traversing Mole" server

//! Crate docs

#![forbid(unsafe_code)]
#![deny(
    missing_docs,
    unstable_features,
    unsafe_code,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
// For development:
#![allow(dead_code, unused_imports, unused_variables, deprecated)]


#[macro_use]
extern crate fern;

use actix::prelude::*;
use chrono::*;
use colored::Colorize;
use curl::{
    easy::{Easy2, List},
    multi::{Easy2Handle, Multi},
    Error as CurlError, MultiError,
};
use fern::InitError;
use krecik::{
    actors::{
        curl_multi_checker::{Checks, CurlMultiChecker},
        curl_multi_checker_pongo::Checks as ChecksPongo,
        curl_multi_checker_pongo::CurlMultiCheckerPongo,
        domain_expiry_checker::Checks as ChecksDomains,
        domain_expiry_checker::DomainExpiryChecker,
        history_teacher::{HistoryTeacher, Results},
        results_warden::{ResultsWarden, ValidateResults},
    },
    api::*,
    checks::{
        check::*,
        domain::Domains,
        generic::*,
        page::{Method, Page},
        pongo::{
            collect_pongo_domains, collect_pongo_hosts, get_pongo_checks, read_pongo_mapper,
        },
    },
    configuration::{
        CHECKS_DIR, CHECK_CONNECTION_TIMEOUT, CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE,
        CHECK_MAX_CONNECTIONS, CHECK_MAX_REDIRECTIONS, CHECK_TIMEOUT, DEFAULT_SLACK_NAME,
        REMOTE_CHECKS_DIR,
    },
    products::{
        expected::{Expected, PageExpectation, PageExpectations},
        story::*,
        unexpected::{Unexpected, UnexpectedMinor},
    },
    utilities::list_all_checks_from,
    *,
};
use log::*;
use rayon::prelude::*;
use ssl_expiration2::SslExpiration;
use std::{
    env::{self, var},
    fs,
    io::{Error, ErrorKind},
    path::Path,
    thread,
    time::Duration,
};


fn setup_logger(level: LevelFilter) -> Result<(), InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}


#[actix_macros::main]
async fn main() {
    let logger_level = match var("DEBUG") {
        Ok(value) => LevelFilter::Debug,
        Err(_) => LevelFilter::Info,
    };
    setup_logger(logger_level).unwrap_or_default();

    // Define system actors
    let curl_multi_checker = SyncArbiter::start(4, || CurlMultiChecker);
    let curl_multi_checker_pongo = SyncArbiter::start(4, || CurlMultiCheckerPongo);
    let history_teacher = SyncArbiter::start(4, || HistoryTeacher);
    let results_warden = SyncArbiter::start(1, || ResultsWarden);

    ctrlc::set_handler(|| {
        println!("\n\nKrecik server was interrupted!");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        // TODO: let config = KrecikConfiguration{…}; => reload configuration every loop iteration
        let start = Local::now();

        let pongo_checks = curl_multi_checker_pongo
            .send(ChecksPongo(all_checks_pongo_merged()))
            .await;

        let regular_checks = curl_multi_checker
            .send(Checks(all_checks_but_remotes()))
            .await;

        let stories = [
            pongo_checks.unwrap().unwrap_or_default(),
            regular_checks.unwrap().unwrap_or_default(),
        ]
        .concat();

        let end = Local::now();
        let diff = end - start;

        info!(
            "Remote checks took: {}s. Result stories count: {}.",
            diff.num_seconds(),
            stories.len(),
        );

        debug!("Sending results to HistoryTeacher…");
        history_teacher
            .send(Results(stories))
            .await
            .unwrap()
            .unwrap_or_default();
        // TODO: HistoryTeacher should send VadlidateResults message after it's done to eliminate possible race condition when stories weren't saved yet and ResultsWarden is already on validation process.

        debug!("Starting results validation…");
        match results_warden.send(ValidateResults).await.unwrap() {
            Ok(()) => {
                // only if no issues were found, we can do a pause between tests
                debug!("No issues detected, pausing before next check…");
                thread::sleep(Duration::from_millis(30_000));
            }
            Err(()) => {
                debug!("Errors were detected, starting next check immediately…");
            }
        }
    }
}
