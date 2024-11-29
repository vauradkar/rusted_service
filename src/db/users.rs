use aide::NoApi;
use anyhow::anyhow;
use async_trait::async_trait;
use axum_login::AuthUser;
use axum_login::AuthnBackend;
use axum_login::UserId;
use password_auth::generate_hash;
use password_auth::verify_password;
use rand::Rng;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use tokio::task;

use crate::db::Db;

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct UpdatePassword {
    old: String,
    new_pw: String,
    new_pw_retype: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    id: i64,
    pub username: String,
    password: String,
}

impl User {
    pub async fn update_password(
        &self,
        passwords: UpdatePassword,
        session: &AuthSession,
    ) -> anyhow::Result<()> {
        session.backend.update_password(self, passwords).await
    }
}

#[derive(Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct Preferences {
    greetings: String,
    dark_mode: bool,
}

impl Preferences {
    pub fn random() -> Self {
        let num = rand::thread_rng().gen_range(0..3);
        let msgs = ["hello", "ನಮಸ್ಕಾರ", "नमस्ते"];

        Self {
            greetings: msgs.get(num % msgs.len()).unwrap().to_string(),
            dark_mode: num % 2 == 0,
        }
    }
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes() // We use the password hash as the auth
                                 // hash--what this means
                                 // is when the user changes their password the
                                 // auth session becomes invalid.
    }
}

// This allows us to extract the authentication fields from forms. We use this
// to authenticate requests with the backend.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    #[allow(dead_code)]
    pub next: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: Db,
}

impl Backend {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    async fn update_password(&self, user: &User, passwords: UpdatePassword) -> anyhow::Result<()> {
        let mut conn = self.db.db.acquire().await?;
        if passwords.new_pw != passwords.new_pw_retype {
            return Err(anyhow!("new passwords don't match"));
        }
        let gen_new = generate_hash(&passwords.new_pw);

        let ret = sqlx::query("update users set password = ? where username = ? and password = ? ")
            .bind(gen_new)
            .bind(&user.username)
            .bind(&user.password)
            .execute(&mut *conn)
            .await?
            .rows_affected();
        if ret == 1 {
            Ok(())
        } else {
            Err(anyhow!("failed to find the user with matching password"))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as("select * from users where username = ? ")
            .bind(creds.username)
            .fetch_optional(&self.db.db)
            .await?;

        // Verifying the password is blocking and potentially slow, so we'll do so via
        // `spawn_blocking`.
        task::spawn_blocking(|| {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| verify_password(creds.password, &user.password).is_ok()))
        })
        .await?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("select * from users where id = ?")
            .bind(user_id)
            .fetch_optional(&self.db.db)
            .await?;

        Ok(user)
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
pub type AuthSessionNoApi = NoApi<AuthSession>;
