/// Actors module

/// Generic trait definition for curl checkers
pub mod curl_generic_checker;

///  sync Easy2+Multi bulk actor
pub mod curl_multi_checker;

/// sync Easy2+Multi+Pongo bulk actor
pub mod curl_multi_checker_pongo;

/// sync TLS certificate expiry check actor
pub mod domain_expiry_checker;

/// store history of checks on disk
pub mod history_teacher;
