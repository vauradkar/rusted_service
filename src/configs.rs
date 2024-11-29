//! The module contain a bunch of global/static configs.
//! These are TODOs that one can change update if they are
//! using the template.

use time::Duration;

/// This is where app's apis will be available.
/// TODO: Change this if this path is busy.
static API_BASE_PATH: &str = "/api";

/// Version of the api.
/// TODO: Change this as needed.
static API_VERSION_STR: &str = "v1";

/// TODO: change this to cool servie name that you are writing
static SERVICE_NAME: &str = "TEMPLATE";

/// TODO: update description of the service
static SERVICE_DESCRIPTION: &str = include_str!("../README.md");

/// This is path where app's apis docs will be available.
/// TODO: Change this if this path is busy.
static DOCS_BASE_PATH: &str = "/docs";

/// A short summary of the serivce
/// TODO: Change this
static SERVICE_SUMMARY: &str =
    "A template service that uses using axum, axum-login, sqlite and aide";

/// How long should a logged-in session persist due to inactivity. After not
/// activity for this long, the user needs to re-login
pub static INACTIVE_SESSION_TIMEOUT: Duration = Duration::days(1);

/// Expired sessions are cleared after at this frequency.
pub static DELETE_EXPIRED_FREQUENCY: tokio::time::Duration = tokio::time::Duration::from_secs(60);

/// Static properties of app/service
pub struct ServiceDetails {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) api_version: &'static str,
    pub(crate) api_base_path: &'static str,
    pub(crate) docs_base_path: &'static str,
}

impl ServiceDetails {
    /// Builds api path from their components
    pub fn api_path(&self) -> String {
        [self.api_base_path, self.api_version, self.name].join("/")
    }
}

impl Default for ServiceDetails {
    fn default() -> Self {
        Self {
            name: SERVICE_NAME,
            description: SERVICE_DESCRIPTION,
            summary: SERVICE_SUMMARY,
            api_version: API_VERSION_STR,
            api_base_path: API_BASE_PATH,
            docs_base_path: DOCS_BASE_PATH,
        }
    }
}
