#[allow(unused_imports, clippy::unit_arg, clippy::assertions_on_constants)]
#[cfg(test)]
mod all_tests {

    // Load all internal modules:
    use curl::easy::{Easy, Easy2, Handler, WriteError};
    use curl::multi::{Easy2Handle, Multi};
    use regex::Regex;

    use ssl_expiration2::SslExpiration;
    use std::io::{Error, ErrorKind};
    use std::time::Duration;

    use crate::checks::domain::*;
    use crate::checks::generic::*;
    use crate::checks::page::*;
    use crate::checks::pongo::*;
    use crate::checks::*;
    use crate::configuration::*;
    use crate::products::expected::*;
    use crate::products::unexpected::*;
    use crate::products::*;
    use crate::utilities::*;
    use crate::*;


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
        assert!(!expiration.is_expired());
        assert!(expiration.days() > 10);
    }


    #[test]
    fn test_curl_basic_test() {
        let mut easy = Easy2::new(CollectorForTests(Vec::new()));
        easy.get(true).unwrap_or_default();
        // easy.verbose(true).unwrap_or_default();
        easy.url("https://www.rust-lang.org/").unwrap_or_default();
        easy.perform().unwrap_or_default();
        assert_eq!(
            easy.response_code().unwrap_or_default(),
            CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE
        );
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

        let mut easy1 = Easy2::new(CollectorForTests(Vec::new()));
        easy1.get(true).unwrap_or_default();
        easy1.follow_location(true).unwrap_or_default();
        // easy1.verbose(true).unwrap_or_default();
        easy1.url(url1).unwrap_or_default();
        easy1.max_connects(10).unwrap_or_default();
        easy1.max_redirections(10).unwrap_or_default();

        let mut easy2 = Easy2::new(CollectorForTests(Vec::new()));
        easy2.get(true).unwrap_or_default();
        easy2.follow_location(true).unwrap_or_default();
        // easy2.verbose(true).unwrap_or_default();
        easy2.url("https://docs.rs/").unwrap_or_default();
        easy2.max_connects(10).unwrap_or_default();
        easy2.max_redirections(10).unwrap_or_default();

        let mut easy3 = Easy2::new(CollectorForTests(Vec::new()));
        easy3.get(true).unwrap_or_default();
        easy3.follow_location(true).unwrap_or_default();
        // easy3.verbose(true).unwrap_or_default();
        easy3
            .url("http://sdfsdfsdfdsfdsfds.pl/")
            .unwrap_or_default();
        easy3.max_connects(10).unwrap_or_default();
        easy3.max_redirections(10).unwrap_or_default();

        let mut multi = Multi::new();
        multi.pipelining(true, true).unwrap_or_default();
        let easy1handle = multi.add2(easy1).unwrap();
        let easy2handle = multi.add2(easy2).unwrap();
        let easy3handle = multi.add2(easy3).unwrap();

        while multi.perform().unwrap_or_default() > 0 {
            multi
                .wait(&mut [], Duration::from_secs(1))
                .unwrap_or_default();
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
        assert!(
            handler1after.response_code().unwrap_or_default()
                == CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE
        );
        assert!(handler1after.download_size().unwrap_or_default() > 0f64);

        let mut handler2after = multi.remove2(easy2handle).unwrap();
        assert!(
            handler2after.response_code().unwrap_or_default()
                == CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE
        );
        assert!(handler2after.download_size().unwrap_or_default() > 0f64);

        let mut handler3after = multi.remove2(easy3handle).unwrap();
        assert!(handler3after.response_code().unwrap_or_default() == 0); // NOTE: 0 since no connection is possible to non existing server
        assert!(handler2after.download_size().unwrap_or_default() > 0f64); // even if connection failed, we sent some bytes

        //multi.close().unwrap_or_default();
    }


    #[test]
    fn test_curl_all_options_test() {
        let mut easy = Easy2::new(CollectorForTests(Vec::new()));
        easy.get(true).unwrap_or_default();
        easy.follow_location(true).unwrap_or_default();
        easy.ssl_verify_peer(true).unwrap_or_default();
        easy.ssl_verify_host(true).unwrap_or_default();
        easy.connect_timeout(Duration::from_secs(30))
            .unwrap_or_default();
        easy.timeout(Duration::from_secs(30)).unwrap_or_default();
        easy.max_connects(10).unwrap_or_default();
        easy.max_redirections(10).unwrap_or_default();

        let url = "http://rust-lang.org/";
        easy.url(&url).unwrap_or_default();
        easy.perform().unwrap_or_default();

        println!("URL: {}", &url);
        println!(
            "Redirect count: {:?}",
            easy.redirect_count().unwrap_or_default()
        );
        // println!("Final URL: {:?}", easy.redirect_url().unwrap_or_default());
        println!(
            "Effective URL: {:?}",
            easy.effective_url().unwrap_or_default()
        );
        println!("Local IPv4: {:?}", easy.local_ip().unwrap_or_default());
        println!("Remote IPv4: {:?}", easy.primary_ip().unwrap_or_default());
        println!(
            "Content type: {:?}",
            easy.content_type().unwrap_or_default()
        );
        println!("Cookies: {:?}", easy.cookies().unwrap());
        println!(
            "TIMINGS: Connect time: {:?}, Name lookup time: {:?}, Redirect time: {:?}, Total time: {:?}",
            easy.connect_time().unwrap_or_default(),
            easy.namelookup_time().unwrap_or_default(),
            easy.redirect_time().unwrap_or_default(),
            easy.total_time().unwrap_or_default()
        );

        assert_eq!(
            easy.response_code().unwrap_or_default(),
            CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE
        );
        let contents = easy.get_ref();
        let raw_page = String::from_utf8_lossy(&contents.0);
        assert!(raw_page.contains("Rust"));
        assert!(raw_page.contains("<meta "));
        assert!(raw_page.contains("<head>"));
        assert!(raw_page.contains("<body>"));
    }


    #[test]
    fn test_filecheck_to_json_serialization() {
        let check = GenCheck {
            domains: Some(vec![Domain {
                name: "nask.pl".to_string(),
                expects: vec![DomainExpectation::ValidExpiryPeriod(
                    CHECK_MINIMUM_DAYS_OF_TLSCERT_VALIDITY,
                )],
            }]),
            pages: Some(vec![Page {
                url: "http://rust-lang.org/".to_string(),
                expects: vec![PageExpectation::ValidCode(
                    CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE,
                )],
                options: Some(PageOptions::default()),
            }]),
            alert_webhook: None,
            alert_channel: None,
        };
        let output = serde_json::to_string(&check).unwrap_or_default();
        println!("Output: {}", output);
        assert!(output.len() > 100);
    }


    #[test]
    fn test_check_json_to_filecheck_deserialization() {
        let check = GenCheck::load("checks/tests/test1.json").unwrap_or_default();
        assert!(check.pages.is_some());
        assert!(check.domains.is_some());
    }


    #[test]
    fn test_domain_check_history_length() {
        let check = GenCheck::load("checks/tests/test1.json").unwrap_or_default();
        let history = GenCheck::check_domains(check.domains);
        println!("TEST1({}): {}", history.length(), history.to_string());
        assert!(history.length() > 0);
        assert!(history.length() == 1);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp.len() > 10);
        assert!(first.success.is_some());
    }


    #[test]
    fn test_page_check_history_length() {
        let check = GenCheck::load("checks/tests/test2.json").unwrap_or_default();
        let history = check.execute("");
        println!("TEST2({}): {}", history.length(), history.to_string());
        assert!(history.length() > 3);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp.len() > 10);
    }


    #[test]
    fn test_redirect_no_follow() {
        let check = GenCheck::load("checks/tests/test3.json").unwrap_or_default();
        let history = check.execute("");
        println!("TEST3({}): {}", history.length(), history.to_string());
        assert!(history.length() > 3);
        history.stories().iter().for_each(|story| {
            assert!(story.success.is_some());
            assert!(story.error.is_none());
        });
    }


    #[test]
    fn test_gibberish_url_check() {
        let check = GenCheck::load("checks/tests/test4.json").unwrap_or_default();
        let history = check.execute("");
        println!("TEST4({}): {}", history.length(), history.to_string());
        assert!(history.length() > 3);
        let first = history.head();
        assert!(first.count == 1);
        assert!(first.timestamp.len() > 10);
        assert!(first.success.is_some());
        assert!(first.error.is_none());
    }


    #[test]
    fn test_page_content_length_check() {
        let check = GenCheck::load("checks/tests/test5.json").unwrap_or_default();
        let page: &Page = &check.clone().pages.unwrap_or_default()[0];
        let options = page.options.clone().unwrap_or_default();
        let cookies = options.cookies;
        let headers = options.headers;
        let history = check.execute("");
        println!("TEST5({}): {}", history.length(), history.to_string());
        assert!(history.length() > 3);
        let first = history.head();
        assert!(headers.is_some());
        assert!(cookies.is_some());
        assert!(cookies.unwrap_or_default().len() == 3);
        assert!(first.count == 1);
        assert!(first.timestamp.len() > 10);
    }


    #[test]
    fn test_agent_check() {
        let check = GenCheck::load("checks/tests/test5.json").unwrap_or_default();
        let page: &Page = &check.pages.unwrap_or_default()[0];
        let options = page.options.clone().unwrap_or_default();
        let agent = options.agent;
        assert!(agent.is_some());
        assert!(agent.unwrap_or_default() == "Krtecek-Underground-Agent");
    }


    #[test]
    fn test_when_everything_is_a_failure_test9() {
        GenCheck::load("checks/tests/test9.json")
            .unwrap_or_default()
            .execute("")
            .stories()
            .iter()
            .for_each(|story| {
                assert!(story.success.is_none());
                assert!(story.error.is_some());
            });
    }


    #[test]
    fn test_parsing_bogus_validators() {
        GenCheck::load("checks/tests/test10.json")
            .and_then(|_check| Ok(assert!(false)))
            .unwrap_or_else(|err| {
                assert!(
                    err.to_string()
                        .contains("unknown variant `ValidMoonFlower`")
                );
            });
    }


    #[test]
    fn test_parsing_invalid_validator_value_type() {
        GenCheck::load("checks/tests/test11.json")
            .and_then(|_check| Ok(assert!(false)))
            .unwrap_or_else(|err| {
                assert!(err.to_string().contains("invalid type: string"));
            });
    }


    #[test]
    fn test_empty_check() {
        GenCheck::load("checks/tests/test12.json")
            .and_then(|check| {
                assert!(check.pages.is_some());
                assert!(check.domains.is_none());
                check.execute("").stories().iter().for_each(|story| {
                    assert!(story.success.is_some());
                    assert!(story.error.is_none());
                });
                Ok(())
            })
            .unwrap_or_else(|_err| assert!(false));
    }


    #[test]
    fn test_file_from_path() {
        let path = "/a/file/somewhere/in/a/woods/of/files/is-my-name.txt";
        assert!(file_name_from_path(path) == "is-my-name.txt");
    }

    // test POST
}
