//
// Default configuration and default values:
//


/// Default project directory:
pub const PROJECT_DIRECTORY: &str = "/Projects/travmole";

/// Checks directory:
pub const CHECKS_DIR: &str = "checks/on";

/// Default log file:
pub const DEFAULT_LOG_FILE: &str = "logs/travmole.log";

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
pub const CHECK_SSL_DAYS_EXPIRATION: i32 = 14;

/// Default successful HTTP code: 200
pub const CHECK_DEFAULT_SUCCESSFUL_HTTP_CODE: u32 = 200;
