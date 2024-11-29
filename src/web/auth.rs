use std::sync::Arc;
use std::sync::Mutex;

use aide::axum::routing::get_with;
use aide::axum::routing::post_with;
use aide::axum::ApiRouter;
use aide::axum::IntoApiResponse;
use aide::transform::TransformOperation;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_jsonschema::Json;

use crate::db::users::AuthSessionNoApi;
use crate::db::users::Credentials;
use crate::state::AppState;

pub fn router(state: Arc<Mutex<AppState>>) -> ApiRouter {
    ApiRouter::new()
        .api_route("/signin", post_with(self::signin, self::signin_docs))
        .api_route("/signout", get_with(self::signout, self::signout_docs))
        .with_state(state)
}

pub async fn signin(
    state: State<Arc<Mutex<AppState>>>,
    mut auth_session: AuthSessionNoApi,
    Json(creds): Json<Credentials>,
) -> impl IntoApiResponse {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    {
        let mut state = state.lock().unwrap();
        state.active_users.insert(creds.username.clone());
    }
    StatusCode::OK.into_response()
}

pub fn signin_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.description("sign-in").response::<201, Json<String>>()
}

pub async fn signout(mut auth_session: AuthSessionNoApi) -> impl IntoApiResponse {
    match auth_session.logout().await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
pub fn signout_docs(op: TransformOperation<'_>) -> TransformOperation<'_> {
    op.description("signout").response::<201, Json<String>>()
}
