/// Actors module

/// Generic trait definition for curl checkers
pub mod generic_checker;

///  sync Easy2+Multi bulk actor
pub mod multi_checker;

/// store history of checks on disk
pub mod history_teacher;

/// read last history states and send Slack notifications if necessary
pub mod results_warden;

/// notificator actor
pub mod notificator;
