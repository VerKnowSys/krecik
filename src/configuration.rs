//
// Default configuration and default values:
//


/// Default project directory:
pub const PROJECT_DIRECTORY: &str = "/Projects/krecik";

/// Default log file:
pub const DEFAULT_LOG_FILE: &str = "logs/krecik.log";

/// Default stdout:
pub const DEFAULT_STDOUT_DEV: &str = "/dev/stdout";

/// Default listen address and port:
pub const LISTEN_ADDRESS: &str = "127.0.0.1:60666";

/// Check timeout in seconds
pub const CHECK_TIMEOUT: u64 = 15;

/// Check connection timeout in seconds
pub const CHECK_CONNECTION_TIMEOUT: u64 = 30;

/// Check max connect attempts
pub const CHECK_MAX_CONNECTIONS: u32 = 10;

/// Check max redirections
pub const CHECK_MAX_REDIRECTIONS: u32 = 10;

/// Minimum SSL certificate validity in days
pub const CHECK_MINIMUM_DAYS_OF_TLSCERT_VALIDITY: i32 = 14;

/// Default successful HTTP code: 200
pub const CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE: u32 = 200;

/// Default minimum length of HTTP content
pub const CHECK_HTTP_MINIMUM_LENGHT: usize = 128;

/// Default page content expectation:
pub const CHECK_DEFAULT_CONTENT_EXPECTATION: &str = "body";

/// Checks directory:
pub const CHECKS_DIR: &str = "checks";

/// Web-API endpoint
pub const CHECK_API_EXECUTE_REQUEST_PATH: &str = "/check/execute";

/// Default Web proto:
pub const CHECK_DEFAULT_PROTOCOL: &str = "https://";

/// Remote Web-API endpoint
pub const CHECK_API_EXECUTE_REMOTE_REQUEST_PATH: &str = "/check/execute_remote";
