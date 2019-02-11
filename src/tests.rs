#[allow(unused_imports)]
#[cfg(test)]
mod tests {

    // Load all internal modules:
    // use hyper::*;
    use gotham::test::TestServer;
    use regex::Regex;
    use ssl_expiration::SslExpiration;

    use curl::easy::{Easy, Easy2, Handler, WriteError};
    use curl::multi::{Easy2Handle, Multi};
    use std::time::Duration;

    use crate::*;
    use crate::web::router;


    struct CollectorForTests(Vec<u8>);

    impl Handler for CollectorForTests {
        fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
            self.0.extend_from_slice(data);
            println!("CHUNK({})", String::from_utf8_lossy(data).len());
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
        let mut easy = Easy2::new(CollectorForTests(Vec::new()));
        easy.get(true).unwrap();
        // easy.verbose(true).unwrap();
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


    #[test]
    fn test_curl_multi_test() {
        let url1 = "https://www.rust-lang.org/";
        let url2 = "https://www.centra.com/";

        let mut easy1 = Easy2::new(CollectorForTests(Vec::new()));
        easy1.get(true).unwrap();
        easy1.follow_location(true).unwrap();
        // easy1.verbose(true).unwrap();
        easy1.url("https://www.rust-lang.org/").unwrap();
        easy1.max_connects(10).unwrap();
        easy1.max_redirections(10).unwrap();

        let mut easy2 = Easy2::new(CollectorForTests(Vec::new()));
        easy2.get(true).unwrap();
        easy2.follow_location(true).unwrap();
        // easy2.verbose(true).unwrap();
        easy2.url("https://docs.rs/").unwrap();
        easy2.max_connects(10).unwrap();
        easy2.max_redirections(10).unwrap();

        let mut easy3 = Easy2::new(CollectorForTests(Vec::new()));
        easy3.get(true).unwrap();
        easy3.follow_location(true).unwrap();
        // easy3.verbose(true).unwrap();
        easy3.url("http://sdfsdfsdfdsfdsfds.pl/").unwrap();
        easy3.max_connects(10).unwrap();
        easy3.max_redirections(10).unwrap();

        let mut multi = Multi::new();
        multi.pipelining(true, true).unwrap();
        let easy1handle = multi.add2(easy1).unwrap();
        let easy2handle = multi.add2(easy2).unwrap();
        let easy3handle = multi.add2(easy3).unwrap();

        while multi.perform().unwrap() > 0 {
            multi.wait(&mut [], Duration::from_secs(1)).unwrap();
        }

        // 1
        let handler1 = easy1handle.get_ref();
        let raw_page = String::from_utf8_lossy(&handler1.0);
        assert!(raw_page.contains("Rust"));
        assert!(raw_page.contains("<meta "));
        assert!(raw_page.contains("<head>"));
        assert!(raw_page.contains("<body>"));

        // 2
        let handler2 = easy2handle.get_ref();
        let raw_page = String::from_utf8_lossy(&handler2.0);
        assert!(raw_page.contains("Docs.rs"));

        // 3
        let handler3 = easy3handle.get_ref();
        let raw_page = String::from_utf8_lossy(&handler3.0);
        assert!(raw_page.len() == 0);

        let mut handler1after = multi.remove2(easy1handle).unwrap();
        assert!(handler1after.response_code().unwrap() == 200);

        let mut handler2after = multi.remove2(easy2handle).unwrap();
        assert!(handler2after.response_code().unwrap() == 200);

        let mut handler3after = multi.remove2(easy3handle).unwrap();
        assert!(handler3after.response_code().unwrap() == 0); // NOTE: 0 since no connection is possible to non existing server

        multi.close().unwrap();
    }


}
