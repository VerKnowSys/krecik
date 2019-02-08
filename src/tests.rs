#[allow(unused_imports)]
#[cfg(test)]
mod tests {

    // Load all internal modules:
    // use hyper::*;
    use gotham::test::TestServer;
    use regex::Regex;
    use ssl_expiration::SslExpiration;
    use curl::easy::{Easy2, Handler, WriteError};
    use crate::*;
    use crate::web::router;


    struct CollectorForTests(Vec<u8>);

    impl Handler for CollectorForTests {
        fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
            self.0.extend_from_slice(data);
            Ok(data.len())
        }
    }


    #[test]
    fn test_ssl_domain_expiration() {
        let domain = "google.com";
        let expiration = SslExpiration::from_domain_name(&domain).unwrap();
        assert!(expiration.is_expired() == false);
        assert!(expiration.days() > 300);
    }


    #[test]
    fn test_curl_basic_test() {
        let domain = "https://www.google.com/";
        let mut easy = Easy2::new(CollectorForTests(Vec::new()));
        easy.get(true).unwrap();
        easy.verbose(true).unwrap();
        easy.url("https://www.rust-lang.org/").unwrap();
        easy.perform().unwrap();
        assert_eq!(easy.response_code().unwrap(), 200);
        let contents = easy.get_ref();
        let raw_page = String::from_utf8_lossy(&contents.0);
        assert!(raw_page.contains("Rust"));
        assert!(raw_page.contains("<meta "));
        assert!(raw_page.contains("<head>"));
        assert!(raw_page.contains("<body>"));
    }
}
