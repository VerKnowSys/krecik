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

use std::{thread, time::Duration};


type TracingEnvFilterHandle =
    Handle<EnvFilter, Layered<Layer<Registry, DefaultFields, Format<Compact>>, Registry>>;


#[instrument]
fn initialize_logger() -> TracingEnvFilterHandle {
    let env_log_filter = match EnvFilter::try_from_env("LOG") {
        Ok(env_value_from_env) => env_value_from_env,
        Err(_) => EnvFilter::from("info"),
    };
    let fmt = fmt()
        .compact()
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_ansi(true)
        .with_env_filter(env_log_filter)
        .with_filter_reloading();

    let handle = fmt.reload_handle();
    fmt.init();
    handle
}


#[instrument]
#[actix_macros::main]
async fn main() {
    let log_reload_handle = initialize_logger();

    addy::mediate(SIGUSR1)
        .register("toggle_log_level", move |_signal| {
            log_reload_handle
                .modify(|env_filter| {
                    if env_filter.to_string() == *"info" {
                        println!("SIGNAL: Enabling DEBUG log level after signal: SIGUSR1");
                        *env_filter = EnvFilter::from("debug");
                    } else if env_filter.to_string() == *"debug" {
                        println!("SIGNAL: Enabling TRACE log level after signal: SIGUSR1");
                        *env_filter = EnvFilter::from("trace");
                    } else if env_filter.to_string() == *"trace" {
                        println!("SIGNAL: Enabling INFO log level after signal: SIGUSR1");
                        *env_filter = EnvFilter::from("info");
                    }
                })
                .unwrap_or_default();
        })
        .expect("Couldn't initialize SIGUSR1 handler")
        .enable()
        .expect("SIGUSR1 handler couldn't be enabled");

    addy::mediate(SIGINT)
        .register("interrupt", |_signal| {
            println!("\n\n{} was interrupted!", env!("CARGO_BIN_NAME"));
            std::process::exit(0);
        })
        .expect("Couldn't initialize SIGINT handler")
        .enable()
        .expect("SIGINT handler couldn't be enabled");

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
