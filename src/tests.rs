#[allow(unused_imports, clippy::unit_arg, clippy::assertions_on_constants)]
#[cfg(test)]
mod all_tests {

    // Load all internal modules:
    use curl::{
        easy::{Easy, Easy2, Handler, WriteError},
        multi::{Easy2Handle, Multi},
    };

    use ssl_expiration2::SslExpiration;
    use std::{
        io::{Error, ErrorKind},
        time::Duration,
    };

    use crate::{
        actors::{generic_checker::GenericChecker, multi_checker::MultiChecker},
        checks::{check::*, domain::*, page::*, pongo::*, *},
        configuration::*,
        products::{expected::*, unexpected::*, *},
        utilities::*,
        *,
    };


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
    fn test_curl_basic_test() -> Result<(), curl::Error> {
        let mut easy = Easy2::new(CollectorForTests(Vec::new()));
        easy.get(true)?;
        // easy.verbose(true)?;
        easy.url("https://www.rust-lang.org/")?;
        easy.perform()?;
        assert_eq!(easy.response_code()?, CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE);
        let contents = easy.get_ref();
        let raw_page = String::from_utf8_lossy(&contents.0);
        assert!(raw_page.contains("Rust"));
        assert!(raw_page.contains("<meta "));
        assert!(raw_page.contains("<head>"));
        assert!(raw_page.contains("<body>"));
        Ok(())
    }


    #[test]
    fn test_curl_multi_test() -> Result<(), curl::Error> {
        let url1 = "https://www.rust-lang.org/";

        let mut easy1 = Easy2::new(CollectorForTests(Vec::new()));
        easy1.get(true)?;
        easy1.follow_location(true)?;
        // easy1.verbose(true)?;
        easy1.url(url1)?;
        easy1.max_connects(10)?;
        easy1.max_redirections(10)?;

        let mut easy2 = Easy2::new(CollectorForTests(Vec::new()));
        easy2.get(true)?;
        easy2.follow_location(true)?;
        // easy2.verbose(true)?;
        easy2.url("https://docs.rs/")?;
        easy2.max_connects(10)?;
        easy2.max_redirections(10)?;

        let mut easy3 = Easy2::new(CollectorForTests(Vec::new()));
        easy3.get(true)?;
        easy3.follow_location(true)?;
        // easy3.verbose(true)?;
        easy3.url("http://sdfsdfsdfdsfdsfds.pl/")?;
        easy3.max_connects(10)?;
        easy3.max_redirections(10)?;

        call_multi_helper(easy1, easy2, easy3).map_err(|_err| {
            assert!(false);
            curl::Error::new(1)
        })
    }

    fn call_multi_helper(
        easy1: Easy2<CollectorForTests>,
        easy2: Easy2<CollectorForTests>,
        easy3: Easy2<CollectorForTests>,
    ) -> Result<(), curl::MultiError> {
        let mut multi = Multi::new();
        multi.pipelining(false, true)?;
        let easy1handle = multi.add2(easy1)?;
        let easy2handle = multi.add2(easy2)?;
        let easy3handle = multi.add2(easy3)?;

        while multi.perform().unwrap() > 0 {
            multi.wait(&mut [], Duration::from_secs(1))?;
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

        let mut handler1after = multi.remove2(easy1handle)?;
        assert!(handler1after.response_code().unwrap() == CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE);
        assert!(handler1after.download_size().unwrap() > 0f64);

        let mut handler2after = multi.remove2(easy2handle)?;
        assert!(handler2after.response_code().unwrap() == CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE);
        assert!(handler2after.download_size().unwrap() > 0f64);

        let mut handler3after = multi.remove2(easy3handle)?;
        assert!(handler3after.response_code().unwrap() == 0); // NOTE: 0 since no connection is possible to non existing server
        assert!(handler2after.download_size().unwrap() > 0f64); // even if connection failed, we sent some bytes

        Ok(())
    }


    #[test]
    fn test_curl_all_options_test() -> Result<(), curl::Error> {
        let mut easy = Easy2::new(CollectorForTests(Vec::new()));
        easy.get(true)?;
        easy.follow_location(true)?;
        easy.ssl_verify_peer(true)?;
        easy.ssl_verify_host(true)?;
        easy.connect_timeout(Duration::from_secs(30))?;
        easy.timeout(Duration::from_secs(30))?;
        easy.max_connects(10)?;
        easy.max_redirections(10)?;

        let url = "http://rust-lang.org/";
        easy.url(&url)?;
        easy.perform()?;

        println!("URL: {}", &url);
        println!("Redirect count: {:?}", easy.redirect_count()?);
        // println!("Final URL: {:?}", easy.redirect_url()?);
        println!("Effective URL: {:?}", easy.effective_url()?);
        println!("Local IPv4: {:?}", easy.local_ip()?);
        println!("Remote IPv4: {:?}", easy.primary_ip()?);
        println!("Content type: {:?}", easy.content_type()?);
        println!("Cookies: {:?}", easy.cookies()?);
        println!(
            "TIMINGS: Connect time: {:?}, Name lookup time: {:?}, Redirect time: {:?}, Total time: {:?}",
            easy.connect_time()?,
            easy.namelookup_time()?,
            easy.redirect_time()?,
            easy.total_time()?
        );

        assert_eq!(easy.response_code()?, CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE);
        let contents = easy.get_ref();
        let raw_page = String::from_utf8_lossy(&contents.0);
        assert!(raw_page.contains("Rust"));
        assert!(raw_page.contains("<meta "));
        assert!(raw_page.contains("<head>"));
        assert!(raw_page.contains("<body>"));

        Ok(())
    }


    #[test]
    fn test_filecheck_to_json_serialization() {
        let check = Check {
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
            notifier: None,
        };
        let output = serde_json::to_string(&check).unwrap();
        println!("Output: {}", output);
        assert!(output.len() > 100);
    }


    #[test]
    fn test_check_json_to_filecheck_deserialization() {
        let check = read_single_check("checks/tests/test1.json").unwrap();
        assert!(check.pages.is_some());
        assert!(check.domains.is_some());
    }


    #[test]
    fn test_domain_check_history_length() {
        let check = read_single_check("checks/tests/test1.json").unwrap();
        let history = MultiChecker::check_domains(&[check]);
        assert_eq!(history.len(), 2);
        for element in history {
            assert!(!element.timestamp.is_empty());
            assert!(element.success.is_some());
            assert!(element.minor.is_none());
            assert!(element.error.is_none());
        }
    }


    #[test]
    fn test_page_check_history_length() {
        let check = read_single_check("checks/tests/test1.json").unwrap();
        let history = MultiChecker::check_pages(&[check]);
        assert_eq!(history.len(), 12);
        for element in history {
            assert!(!element.timestamp.is_empty());
            assert!(element.success.is_some());
            assert!(element.minor.is_none());
            assert!(element.error.is_none());
        }
    }


    #[test]
    fn test_single_page_check_history_length() {
        let check = read_single_check("checks/tests/test2.json").unwrap();
        let history = MultiChecker::check_pages(&[check]);
        assert_eq!(history.len(), 3);
        for element in history {
            assert!(!element.timestamp.is_empty());
            assert!(element.success.is_some());
            assert!(element.minor.is_none());
            assert!(element.error.is_none());
        }
    }


    #[test]
    fn test_redirect_no_follow() {
        let check = read_single_check("checks/tests/test3.json").unwrap();
        let history = MultiChecker::check_pages(&[check]);
        assert_eq!(history.len(), 3);
        for story in history {
            assert!(story.success.is_some());
            assert!(story.minor.is_none());
            assert!(story.error.is_none());
        }
    }


    #[test]
    fn test_gibberish_url_check() {
        let check = read_single_check("checks/tests/test4.json").unwrap();
        let history = MultiChecker::check_pages(&[check]);
        assert_eq!(history.len(), 1);
        for story in history {
            assert!(story.success.is_none());
            assert!(story.minor.is_none());
            assert!(story.error.is_some());
        }
    }


    #[test]
    fn test_page_check_options_in_check() {
        let check = read_single_check("checks/tests/test5.json").unwrap();
        let page: &Page = &check.clone().pages.unwrap()[0];
        let options = page.options.clone().unwrap();
        let cookies = options.cookies;
        let headers = options.headers;
        let history = MultiChecker::check_pages(&[check]);
        assert_eq!(history.len(), 3);
        assert!(headers.is_some());
        assert!(cookies.is_some());
        assert_eq!(cookies.unwrap().len(), 3);
    }


    #[test]
    fn test_agent_check() {
        let check = read_single_check("checks/tests/test5.json").unwrap();
        let page: &Page = &check.pages.unwrap()[0];
        let options = page.options.clone().unwrap();
        let agent = options.agent;
        assert!(agent.is_some());
        assert_eq!(agent.unwrap(), "Krtecek-Underground-Agent");
    }


    #[test]
    fn test_when_everything_is_a_failure_test9() {
        let check = read_single_check("checks/tests/test9.json").unwrap();
        MultiChecker::check_pages(&[check])
            .iter()
            .for_each(|story| {
                assert!(story.success.is_none());
                assert!(story.minor.is_some() || story.error.is_some()); // validation check for undefined domain is minor not error
            });
    }


    #[test]
    fn test_parsing_bogus_validators() {
        match read_single_check("checks/tests/test10.json") {
            Some(_) => assert!(false),
            None => assert!(true),
        }
    }


    #[test]
    fn test_parsing_bogus_validators_with_result() {
        read_single_check_result("checks/tests/test10.json")
            .map(|_check| assert!(false))
            .unwrap_or_else(|err| {
                assert!(
                    err.to_string()
                        .contains("unknown variant `ValidMoonFlower`")
                );
            });
    }


    #[test]
    fn test_parsing_invalid_validator_value_type() {
        match read_single_check("checks/tests/test11.json") {
            Some(_) => assert!(false),
            None => assert!(true),
        }
    }


    #[test]
    fn test_parsing_invalid_validator_value_type_with_result() {
        read_single_check_result("checks/tests/test11.json")
            .map(|_check| assert!(false))
            .unwrap_or_else(|err| {
                assert!(err.to_string().contains("invalid type: string"));
            });
    }


    #[test]
    fn test_bogus_formatted_check() {
        match read_single_check("checks/tests/test11.json") {
            Some(_) => assert!(false),
            None => assert!(true),
        }
    }


    #[test]
    fn test_file_from_path() {
        let path = "/a/file/somewhere/in/a/woods/of/files/is-my-name.txt";
        assert!(file_name_from_path(path) == "is-my-name.txt");
    }

    // test POST
}
