//
// Default configuration and default values:
//


/// Default log file:
pub const DEFAULT_LOG_FILE: &str = "krecik.log";

/// Default Notification name:
pub const DEFAULT_SLACK_NAME: &str = "Krecik";

/// Default Notification failure icon:
pub const DEFAULT_SLACK_FAILURE_ICON: &str = ":error:";

/// Default Notification success icon:
pub const DEFAULT_SLACK_SUCCESS_ICON: &str = ":white_check_mark:";

/// Default failure notification color:
pub const DEFAULT_SLACK_FAILURE_COLOR: &str = "#ff1111";

/// Default success notification color:
pub const DEFAULT_SLACK_SUCCESS_COLOR: &str = "#00ff00";

/// Check timeout in seconds
pub const CHECK_TIMEOUT: u64 = 10;

/// Check connection timeout in seconds
pub const CHECK_CONNECTION_TIMEOUT: u64 = 15;

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

/// Default interval (in seconds) between checks to not exceed 3 checks per minute (considered a flood)
pub const CHECK_MINIMUM_INTERVAL: usize = 20;

/// Default page content expectation:
pub const CHECK_DEFAULT_CONTENT_EXPECTATION: &str = "body";

/// Default successful notification content
pub const CHECK_DEFAULT_SUCCESS_NOTIFICATION_MSG: &str = "All services are UP!";

/// Checks directory:
pub const CHECKS_DIR: &str = "checks";

/// Remote checks directory:
pub const REMOTE_CHECKS_DIR: &str = "remotes";

/// Tests directory:
pub const TESTS_DIR: &str = "tests";

/// Default Web proto:
pub const CHECK_DEFAULT_PROTOCOL: &str = "https://";
