#[allow(unused_imports)]
#[cfg(test)]
mod tests {

    // Load all internal modules:
    use gotham::test::TestServer;
    use regex::Regex;
    use ssl_expiration::SslExpiration;
    use std::io::{Error, ErrorKind};
    use curl::easy::{Easy, Easy2, Handler, WriteError};
    use curl::multi::{Easy2Handle, Multi};
    use std::time::Duration;
    use serde_json;

    use crate::*;
    use crate::configuration::*;
    use crate::utilities::*;
    use crate::inputs::file::*;
    use crate::checks::domain::*;
    use crate::inputs::check::*;
    use crate::checks::page::*;
    use crate::products::*;
    use crate::products::expected::*;
    use crate::products::unexpected::*;
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
        assert!(handler1after.download_size().unwrap() > 0f64);

        let mut handler2after = multi.remove2(easy2handle).unwrap();
        assert!(handler2after.response_code().unwrap() == 200);
        assert!(handler2after.download_size().unwrap() > 0f64);

        let mut handler3after = multi.remove2(easy3handle).unwrap();
        assert!(handler3after.response_code().unwrap() == 0); // NOTE: 0 since no connection is possible to non existing server
        assert!(handler2after.download_size().unwrap() > 0f64); // even if connection failed, we sent some bytes

        multi.close().unwrap();
    }


    #[test]
    fn test_curl_all_options_test() {
        let mut easy = Easy2::new(CollectorForTests(Vec::new()));
        easy.get(true).unwrap();
        easy.follow_location(true).unwrap();
        easy.ssl_verify_peer(true).unwrap();
        easy.ssl_verify_host(true).unwrap();
        easy.connect_timeout(Duration::from_secs(30)).unwrap();
        easy.timeout(Duration::from_secs(30)).unwrap();
        easy.max_connects(10).unwrap();
        easy.max_redirections(10).unwrap();

        let url = "http://rust-lang.org/";
        easy.url(&url).unwrap();
        easy.perform().unwrap();

        println!("URL: {}", &url);
        println!("Redirect count: {:?}", easy.redirect_count().unwrap());
        println!("Final URL: {:?}", easy.redirect_url().unwrap());
        println!("Local IPv4: {:?}", easy.local_ip().unwrap());
        println!("Remote IPv4: {:?}", easy.primary_ip().unwrap());
        println!("Content type: {:?}", easy.content_type().unwrap());
        println!("Cookies: {:?}", easy.cookies().unwrap());
        println!("TIMINGS: Connect time: {:?}, Name lookup time: {:?}, Redirect time: {:?}, Total time: {:?}",
                 easy.connect_time().unwrap(), easy.namelookup_time().unwrap(), easy.redirect_time().unwrap(), easy.total_time().unwrap());

        assert_eq!(easy.response_code().unwrap(), 200);
        let contents = easy.get_ref();
        let raw_page = String::from_utf8_lossy(&contents.0);
        assert!(raw_page.contains("Rust"));
        assert!(raw_page.contains("<meta "));
        assert!(raw_page.contains("<head>"));
        assert!(raw_page.contains("<body>"));
    }


    #[test]
    fn test_filecheck_to_json_serialization() {
        let check = FileCheck {
            name: Some("Testcheck".to_string()),
            domains: Some(vec!(
               Domain {
                   name: "rust-lang.org".to_string(),
                   expects: Some(vec!(DomainExpectation::ValidResolvable,
                                      DomainExpectation::ValidExpiryPeriod(14))),
               }
            )),
            pages: Some(vec!(
                Page {
                    url: "http://rust-lang.org/".to_string(),
                    expects: Some(vec!(PageExpectation::ValidCode(200))),
                    options: Some(PageOptions::default())
                }
            )),
            alert_webhook: None,
            alert_channel: None,
        };
        let output = serde_json::to_string(&check).unwrap();
        println!("Output: {}", output.to_string());
        assert!(output.len() > 100);
    }


    #[test]
    fn test_check_json_to_filecheck_deserialization() {
        let check = FileCheck::load("tests/test1").unwrap();
        assert!(check.name.unwrap() == "Testcheck");
    }


    #[test]
    fn test_domain_check_history_length() {
        let check = FileCheck::load("tests/test1").unwrap();
        let history = FileCheck::check_domains(check.domains).unwrap();
        assert!(history.length() > 0);
        assert!(history.length() == 1);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp > 1550287754);
        assert!(first.message.clone().unwrap_or_default().contains("is valid for "));
    }


    #[test]
    fn test_page_check_history_length() {
        let check = FileCheck::load("tests/test2").unwrap();
        let history = check.execute().unwrap();
        assert!(history.length() == 1);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp > 1550287754);
        assert!(first.message.clone().unwrap_or_default().contains("Got expected "));
    }


    #[test]
    fn test_redirect_no_follow() {
        let check = FileCheck::load("tests/test3").unwrap();
        let history = check.execute().unwrap();
        assert!(history.length() == 1);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp > 1550287754);
    }


    #[test]
    fn test_gibberish_url_check() {
        let check = FileCheck::load("tests/test4").unwrap();
        let history = check.execute().unwrap();
        assert!(history.length() == 1);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp > 1550287754);
    }


    #[test]
    fn test_page_content_length_check() {
        let check = FileCheck::load("tests/test5").unwrap();
        let history = check.execute().unwrap();
        assert!(history.length() == 1);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp > 1550287754);
    }

}
