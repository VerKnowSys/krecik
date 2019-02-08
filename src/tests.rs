#[allow(unused_imports)]
#[cfg(test)]
mod tests {

    // Load all internal modules:
    use hyper::*;
    use gotham::test::TestServer;
    use regex::Regex;
    use ssl_expiration::SslExpiration;
    use curl::easy::{Easy2, Handler, WriteError};
    use crate::*;
    use crate::web::router;


    #[test]
    fn test_ssl_domain_expiration() {
        let domain = "google.com";
        let expiration = SslExpiration::from_domain_name(&domain).unwrap();
        assert!(expiration.is_expired() == false);
        assert!(expiration.days() > 300);
    }


}
