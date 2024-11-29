use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    /// name of the app instance
    pub name: String,

    /// Path where db can be found or created
    pub db_path: String,

    /// Path where app data can be stored
    pub data_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "axum, axum-login, sqlx, aide template".to_owned(),
            db_path: "./sqlite.db".to_owned(),
            data_path: "./cache".to_owned(),
        }
    }
}
