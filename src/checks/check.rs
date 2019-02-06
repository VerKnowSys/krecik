use crate::checks::domain::*;
use crate::checks::page::*;
use crate::products::expected::*;
use crate::products::unexpected::*;


trait Check {

    /// Prepare domains
    fn domains() -> Domains;

    /// Prepare pages
    fn pages() -> Pages;

    /// Prepare domain expectations for check
    fn domain_expectations() -> DomainExpectations;

    /// Prepare page expectations for check
    fn page_expectations() -> PageExpectations;

    /// Alert channel
    fn alert_channel() -> String;

    /// Notification channel
    fn notify_channel() -> String;

    /// Execute check, produce history
    fn execute() -> History;

}
