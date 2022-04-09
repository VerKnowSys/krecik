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
    unused_qualifications,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]

// For development:
// #![allow(dead_code, unused_imports, unused_variables, deprecated)]

use actix::prelude::*;
use addy::Signal::*;
use chrono::Local;
use tracing_subscriber::{
    // filter,
    fmt::{
        format::{Compact, DefaultFields, Format},
        Layer, *,
    },
    layer::Layered,
    // registry,
    reload::*,
    EnvFilter,
    Registry,
};

use krecik::{
    actors::{
        history_teacher::{HistoryTeacher, Results},
        multi_checker::{Checks, MultiChecker},
        results_warden::ResultsWarden,
    },
    api::*,
    *,
};

lazy_static! {
    static ref LOG_LEVEL: RwLock<LevelFilter> = RwLock::new(LevelFilter::Info);
}


/// Set log level dynamically at runtime
fn set_log_level() {
    let level = Config::load().get_log_level();
    match LOG_LEVEL.read() {
        Ok(loglevel) => {
            if level != *loglevel {
                drop(loglevel);
                match LOG_LEVEL.write() {
                    Ok(mut log) => {
                        println!("Changing log level to: {}", level);
                        *log = level
                    }
                    Err(err) => {
                        eprintln!("Failed to change log level to: {}, cause: {}", level, err);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Couldn't read LOG_LEVEL, cause: {:?}", err);
        }
    }
}


#[instrument]
fn setup_logger() -> Result<(), SetLoggerError> {
    let log_file = Config::load()
        .log_file
        .unwrap_or_else(|| String::from(DEFAULT_LOG_FILE));
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::Magenta)
        .trace(Color::Cyan);
    Dispatch::new()
        .filter(|metadata| {
            match LOG_LEVEL.read() {
                Ok(log) => metadata.level() <= *log,
                Err(_err) => true,
            }
        })
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
        .chain(std::io::stdout())
        .chain(fern::DateBased::new(format!("{}.", log_file), "%Y-%m-%d"))
        .apply()
}


#[instrument]
#[actix_macros::main]
async fn main() {
    setup_logger().expect("Couldn't initialize logger");
    ctrlc::set_handler(|| {
        println!("\n\nKrecik server was interrupted!");
        std::process::exit(0);
    })
    .expect("Couldn't initialize Ctrl-C handler");

    // TODO: implement validation of all defined checks using read_single_check_result()
    info!(
        "Starting {} version {}",
        env!("CARGO_BIN_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // Define system actors
    let num = 1;
    let multi_checker = SyncArbiter::start(num, || MultiChecker);
    let history_teacher = SyncArbiter::start(num, || HistoryTeacher);
    let results_warden = SyncArbiter::start(num, || ResultsWarden);
    let notificator = SyncArbiter::start(num, || Notificator);

    loop {
        set_log_level();
        debug!("New execution iteration…");

        let start = Local::now();

        let all_checks = [all_checks_pongo_merged(), all_checks_but_remotes()].concat();
        if all_checks.is_empty() {
            warn!(
                "No checks defined under root dir: '{}'! Iteration skipped…",
                format!(
                    "{}/{}",
                    Config::load()
                        .krecik_root
                        .unwrap_or_else(|| ".".to_string()),
                    CHECKS_DIR
                )
            );
            thread::sleep(Duration::from_secs(60));
        } else {
            let stories = multi_checker
                .send(Checks(all_checks))
                .await
                .unwrap()
                .unwrap_or_default();

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

            // Let's make sure Krecik is not flooding with checks
            if diff.num_seconds() < CHECK_MINIMUM_INTERVAL as i64 {
                debug!("Throttling next iteration by {CHECK_MINIMUM_INTERVAL}s");
                thread::sleep(Duration::from_secs(CHECK_MINIMUM_INTERVAL as u64));
            }
        }
    }
}
