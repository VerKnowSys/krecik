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
pub const LISTEN_ADDRESS: &str = "172.16.1.15:60666";

/// Check timeout in seconds
pub const CHECK_TIMEOUT: u64 = 15;

/// Check connection timeout in seconds
pub const CHECK_CONNECTION_TIMEOUT: u64 = 30;
