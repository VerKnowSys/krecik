//! "Traversing Mole" server

//! Crate docs

#![forbid(unsafe_code)]
#![deny(
    missing_docs,
    unstable_features,
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
use curl::{
    easy::{Easy2, List},
    multi::{Easy2Handle, Multi},
    Error as CurlError, MultiError,
};
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch, InitError,
};
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


fn setup_logger(level: LevelFilter) -> Result<(), SetLoggerError> {
    let log_file = Config::load()
        .log_file
        .unwrap_or_else(|| String::from("krecik.log"));
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::Magenta)
        .trace(Color::Cyan);
    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                target = record.target(),
                level = record.level(),
                message = message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .chain(fern::DateBased::new(format!("{}.", log_file), "%Y-%m-%d"))
        .apply()
}


#[actix_macros::main]
async fn main() {
    let logger_level = match var("DEBUG") {
        Ok(value) => {
            if value == *"2" {
                LevelFilter::Trace
            } else {
                LevelFilter::Debug
            }
        }
        Err(_) => LevelFilter::Info, /* TODO: read debug value from configuration and dynamically setup debug logging: */
    };
    setup_logger(logger_level).unwrap_or_default();

    ctrlc::set_handler(|| {
        println!("\n\nKrecik server was interrupted!");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let num = num_cpus::get();
    info!(
        "Starting Krecik-server v{} with {} threads per check-actor.",
        env!("CARGO_PKG_VERSION"),
        num
    );

    // Define system actors
    let curl_multi_checker = SyncArbiter::start(num, || CurlMultiChecker);
    let curl_multi_checker_pongo = SyncArbiter::start(num, || CurlMultiCheckerPongo);
    let history_teacher = SyncArbiter::start(num, || HistoryTeacher);
    let results_warden = SyncArbiter::start(num, || ResultsWarden);
    let notificator = SyncArbiter::start(num, || Notificator);

    loop {
        debug!("New execution iteration…");

        let start = Local::now();

        let pongo_checks = &curl_multi_checker_pongo
            .send(ChecksPongo(all_checks_pongo_merged()))
            .await;

        let regular_checks = curl_multi_checker
            .send(Checks(all_checks_but_remotes()))
            .await;
        let stories = [
            pongo_checks.clone().unwrap().unwrap_or_default(),
            regular_checks.unwrap().unwrap_or_default(),
        ]
        .concat();

        let end = Local::now();
        let diff = end - start;

        warn_for_undefined_notifiers(&stories);

        info!(
            "Remote checks took: {}s. Result stories count: {}.",
            diff.num_seconds(),
            stories.len(),
        );

        debug!("Sending results to HistoryTeacher…");
        history_teacher
            .send(Results(
                stories,
                results_warden.clone(),
                notificator.clone(),
            ))
            .await
            .unwrap_or_default();
    }
}
