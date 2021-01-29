use crate::{
    checks::check::*,
    configuration::CHECK_TIMEOUT,
    products::{
        expected::{DomainExpectation, Expected},
        story::*,
        unexpected::{Unexpected, UnexpectedMinor},
    },
};
use actix::prelude::*;
use rayon::prelude::*;
use ssl_expiration2::SslExpiration;


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
            .collect())
    }
}


impl DomainExpiryChecker {
    /// Check SSL certificate expiration using OpenSSL function
    fn check_ssl_expire(
        domain_name: &str,
        domain_expectation: DomainExpectation,
        notifier: Option<String>,
    ) -> Story {
        SslExpiration::from_domain_name_with_timeout(&domain_name, CHECK_TIMEOUT)
            .map(|ssl_validator| {
                match domain_expectation {
                    DomainExpectation::ValidExpiryPeriod(expected_days)
                        if ssl_validator.days() < expected_days
                            || ssl_validator.is_expired() =>
                    {
                        Story::error(
                            Unexpected::TLSDomainExpired(
                                domain_name.to_string(),
                                ssl_validator.days(),
                            ),
                            notifier,
                        )
                    }

                    DomainExpectation::ValidExpiryPeriod(expected_days) => {
                        Story::success(
                            Expected::TLSCertificateFresh(
                                domain_name.to_string(),
                                ssl_validator.days(),
                                expected_days,
                            ),
                            notifier,
                        )
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
