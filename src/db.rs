//! Module contains and encapsulates all interactions with database
//!

use anyhow::anyhow;
use anyhow::Result;
use axum_login::tower_sessions::ExpiredDeletion;
use axum_login::tower_sessions::Expiry;
use axum_login::tower_sessions::SessionManagerLayer;
use sqlx::migrate::MigrateDatabase;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use tokio::task::JoinHandle;
use tower_sessions::cookie::Key;
use tower_sessions::service::SignedCookie;
use tower_sessions_sqlx_store::SqliteStore;

use crate::configs::DELETE_EXPIRED_FREQUENCY;
use crate::configs::INACTIVE_SESSION_TIMEOUT;

pub mod users;

#[derive(Debug, Clone)]
pub struct Db {
    pub(super) db: SqlitePool,
}

impl Db {
    pub async fn connect(db_path: &str) -> Result<Self> {
        let db = SqlitePool::connect(db_path).await?;
        sqlx::migrate!().run(&db).await?;

        Ok(Self { db })
    }

    pub async fn create(db_path: &str) -> Result<Self> {
        Sqlite::create_database(db_path)
            .await
            .map_err(|e| anyhow!("failed to create db. Err:{}", e.to_string()))?;
        Self::connect(db_path).await
    }

    // Session layer.
    //
    // This uses `tower-sessions` to establish a layer that will provide the session
    // as a request extension.
    pub async fn create_session(
        &self,
    ) -> Result<(
        JoinHandle<std::result::Result<(), tower_sessions::session_store::Error>>,
        SessionManagerLayer<SqliteStore, SignedCookie>,
    )> {
        let session_store = SqliteStore::new(self.db.clone());
        session_store.migrate().await?;

        // Generate a cryptographic key to sign the session cookie.
        let key = Key::generate();

        let deletion_task = tokio::task::spawn(
            session_store
                .clone()
                .continuously_delete_expired(DELETE_EXPIRED_FREQUENCY),
        );

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(INACTIVE_SESSION_TIMEOUT))
            .with_signed(key);

        Ok((deletion_task, session_layer))
    }
}
