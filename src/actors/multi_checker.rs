use super::generic_checker::GenericChecker;
use crate::{checks::check::*, products::story::*};
use actix::prelude::*;


/// CurlMultiChecker actor for Multi bulk checks (Curl and OpenSSL)
#[derive(Debug, Copy, Clone)]
pub struct MultiChecker;


/// Wrapper for list of Checks
#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<Stories, Stories>")]
pub struct Checks(pub Vec<Check>);


impl Handler<Checks> for MultiChecker {
    type Result = Result<Stories, Stories>;

    fn handle(&mut self, checks: Checks, _ctx: &mut Self::Context) -> Self::Result {
        let stories_from_domains = Self::check_domains(&checks.0);
        let stories_from_pages = Self::check_pages(&checks.0);
        Ok([stories_from_domains, stories_from_pages].concat())
    }
}


impl Actor for MultiChecker {
    type Context = SyncContext<Self>;
}


impl GenericChecker for MultiChecker {}
