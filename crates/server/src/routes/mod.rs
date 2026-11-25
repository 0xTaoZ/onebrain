use axum::{Router, extract::connect_info::IntoMakeServiceWithConnectInfo, routing::get};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::{DeploymentImpl, middleware};

pub mod admin;
pub mod agents;
pub mod analytics;
pub mod approvals;
pub mod browser_lifecycle;
pub mod build_stats;
pub mod capabilities;
pub mod chat;
pub mod config;
pub mod events;
pub mod filesystem;
pub mod frontend;
pub mod github;
pub mod health;
pub mod images;
pub mod inbox;
pub mod onboarding;
pub mod output_validation;
pub mod project_github;
pub mod project_source_control;
pub mod projects;
pub mod scratch;
pub mod tags;
pub mod version;
pub mod workflow;

pub fn router(
    deployment: DeploymentImpl,
) -> IntoMakeServiceWithConnectInfo<Router, std::net::SocketAddr> {
    let app = application(deployment);
    let controller = crate::lan::LanController::new(app.clone());
    app.merge(crate::lan::control_routes(controller))
        .into_make_service_with_connect_info::<std::net::SocketAddr>()
}

pub(crate) fn application(deployment: DeploymentImpl) -> Router {
    // Create routers with different middleware layers
    let base_routes = Router::new()
        .route("/health", get(health::health_check))
        .merge(browser_lifecycle::router())
        .merge(config::router())
        .merge(capabilities::router())
        .merge(chat::router(&deployment))
        .merge(tags::router(&deployment))
        .merge(filesystem::router())
        .merge(events::router(&deployment))
        .merge(agents::router())
        .merge(approvals::router())
        .merge(projects::router())
        .merge(github::router())
        .merge(project_github::router())
        .merge(project_source_control::router())
        .merge(scratch::router(&deployment))
        .merge(workflow::router())
        .merge(onboarding::router())
        .merge(output_validation::router())
        .merge(version::router())
        .merge(analytics::router())
        .merge(build_stats::router())
        .merge(inbox::router())
        .merge(admin::router())
        .nest("/images", images::routes())
        .layer(ValidateRequestHeaderLayer::custom(
            middleware::validate_origin,
        ))
        .with_state(deployment);

    Router::new()
        .route("/", get(frontend::serve_frontend_root))
        .route("/{*path}", get(frontend::serve_frontend))
        .nest("/api", base_routes)
}
