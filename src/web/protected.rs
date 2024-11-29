use std::sync::Arc;
use std::sync::Mutex;

use aide::axum::routing::put_with;
use aide::axum::ApiRouter;
use aide::axum::IntoApiResponse;
use aide::transform::TransformOperation;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::db::users::AuthSessionNoApi;
use crate::db::users::Preferences;
use crate::db::users::UpdatePassword;
use crate::state::AppState;
use crate::web::docs::Json;

pub fn router(state: Arc<Mutex<AppState>>) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/user/config",
            put_with(put_config, put_config_docs).get_with(get_config, get_config_docs),
        )
        .api_route(
            "/user/passwd",
            put_with(update_password, update_password_docs),
        )
        .with_state(state)
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
struct UserDetails {
    messages: Vec<String>,
    username: String,
    preferences: Preferences,
}

/// Get protected
pub async fn get_config(
    State(state): State<Arc<Mutex<AppState>>>,
    auth_session: AuthSessionNoApi,
) -> impl IntoApiResponse {
    {
        let mut state = state.lock().unwrap();
        state.request_count += 1;
    }
    let username = if let Some(user) = &auth_session.user {
        user.username.to_string()
    } else {
        panic!("this module/api should be protected");
    };
    Json(UserDetails {
        messages: vec!["new config".to_owned()],
        username,
        preferences: Preferences::random(),
    })
    .into_response()
}

fn get_config_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.description("Returns a current user config.")
        .response::<201, Json<UserDetails>>()
}

/// Get protected
async fn put_config(
    State(state): State<Arc<Mutex<AppState>>>,
    auth_session: AuthSessionNoApi,
    Json(preferences): Json<Preferences>,
) -> impl IntoApiResponse {
    {
        let mut state = state.lock().unwrap();
        state.request_count += 1;
    }
    if let Some(user) = &auth_session.user {
        println!("setting configs for {}", user.username);
    }

    Json(preferences).into_response()
}

fn put_config_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.description("Stores user config.")
        .response::<201, Json<Preferences>>()
}

/// Update user password
async fn update_password(
    State(state): State<Arc<Mutex<AppState>>>,
    auth_session: AuthSessionNoApi,
    Json(passwords): Json<UpdatePassword>,
) -> impl IntoApiResponse {
    {
        let mut state = state.lock().unwrap();
        state.request_count += 1;
    }
    let user = if let Some(user) = &auth_session.user {
        user
    } else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    user.update_password(passwords, &auth_session)
        .await
        .map_err(|e| {
            println!("{}", e);
            StatusCode::NOT_FOUND
        })
        .into_response()
}

fn update_password_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.description("Updates user password.")
        .response::<201, Json<Preferences>>()
}
