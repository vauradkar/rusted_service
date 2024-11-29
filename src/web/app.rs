use std::sync::Arc;
use std::sync::Mutex;

use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use axum::Extension;
use axum_login::login_required;
use axum_login::AuthManagerLayerBuilder;
use axum_messages::MessagesManagerLayer;
use tokio::signal;
use tokio::task::AbortHandle;

use super::docs::api_docs;
use super::docs::docs_routes;
use crate::args::AppConfig;
use crate::configs::ServiceDetails;
use crate::db::users::Backend;
use crate::db::Db;
use crate::state::AppState;
use crate::web::auth;
use crate::web::protected;

pub struct App {
    db: Db,
    config: AppConfig,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db: Db::create(&config.db_path).await?,
            config,
        })
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let (deletion_task, session_layer) = self.db.create_session().await?;

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(self.db);
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let app_details = ServiceDetails::default();

        let state = Arc::new(Mutex::new(AppState::default()));
        let mut api = OpenApi::default();
        let api_path = app_details.api_path();
        let router = ApiRouter::new()
            .nest(&api_path, protected::router(state.clone()))
            // All the paths that require login sessions should go above this line.
            .route_layer(login_required!(Backend))
            // Following path allows unauthorized access - ex: login, docs, etc
            .merge(auth::router(state.clone()))
            // Doc routes
            .nest_api_service(app_details.docs_base_path, docs_routes(state.clone()))
            // "Build" docs. The routes added after this line won't show up in docs
            .finish_api_with(&mut api, |api| {
                api_docs(api, app_details, self.config.name.clone())
            })
            .layer(MessagesManagerLayer)
            .layer(auth_layer)
            .layer(Extension(Arc::new(api))); // Arc is very important here or you will face massive memory and performance issues;

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        // Ensure we use a shutdown signal to abort the deletion task.
        axum::serve(listener, router.into_make_service())
            .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
            .await?;

        deletion_task.await??;

        Ok(())
    }
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // untested on windows/fuchsia
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}
