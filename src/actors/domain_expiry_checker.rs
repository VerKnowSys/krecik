use crate::{
    api::*,
    checks::{
        domain::{default_domain_expectations, Domains},
        page::Method,
        pongo::{PongoCheck, PongoChecks},
    },
    configuration::{
        CHECKS_DIR, CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE, CHECK_MINIMUM_DAYS_OF_TLSCERT_VALIDITY,
    },
    products::{
        expected::{
            DomainExpectation, DomainExpectations, Expected, PageExpectation, PageExpectations,
        },
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


/// DomainExpiryChecker actor for TLS certificatea expiration check
#[derive(Debug, Copy, Clone)]
pub struct DomainExpiryChecker;


impl Actor for DomainExpiryChecker {
    type Context = SyncContext<Self>;
}


/// Actor message wrapper structure
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<Stories, Stories>")]
pub struct Checks(pub Vec<Check>);


impl Handler<Checks> for DomainExpiryChecker {
    type Result = Result<Stories, Stories>;

    fn handle(&mut self, msg: Checks, _ctx: &mut Self::Context) -> Self::Result {
        Ok(msg
            .0
            .into_par_iter()
            .flat_map(|check| {
                check
                    .domains
                    .par_iter()
                    .flat_map(move |domains| {
                        domains
                            .par_iter()
                            .flat_map(|domain| {
                                domain
                                    .expects
                                    .par_iter()
                                    .map(|expectation| {
                                        Self::check_ssl_expire(&domain.name, *expectation)
                                    })
                                    .collect::<Stories>()
                            })
                            .collect::<Stories>()
                    })
                    .collect::<Stories>()
            })
            .collect())
    }
}


impl DomainExpiryChecker {
    /// Check SSL certificate expiration using OpenSSL function
    fn check_ssl_expire(domain_name: &str, domain_expectation: DomainExpectation) -> Story {
        SslExpiration::from_domain_name_with_timeout(&domain_name, CHECK_TIMEOUT)
            .map(|ssl_validator| {
                match domain_expectation {
                    DomainExpectation::ValidExpiryPeriod(expected_days)
                        if ssl_validator.days() < expected_days
                            || ssl_validator.is_expired() =>
                    {
                        Story::error(Unexpected::TLSDomainExpired(
                            domain_name.to_string(),
                            ssl_validator.days(),
                        ))
                    }

                    DomainExpectation::ValidExpiryPeriod(expected_days) => {
                        Story::success(Expected::TLSCertificateFresh(
                            domain_name.to_string(),
                            ssl_validator.days(),
                            expected_days,
                        ))
                    }
                }
            })
            .unwrap_or_else(|err| {
                Story::minor(UnexpectedMinor::InternalProtocolProblem(
                    domain_name.to_string(),
                    err.to_string(),
                ))
            })
    }
}
