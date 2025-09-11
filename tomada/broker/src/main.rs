use std::net::Ipv4Addr;

use axum::{body::Body, http::Request};
use broker::{Broker, SharedState, api};
use tokio::{net::TcpListener, select};
use tower_http::trace::TraceLayer;
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[derive(OpenApi)]
    #[openapi()]
    struct ApiDoc;

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(LevelFilter::DEBUG)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    tracing::info!("Logging started.");

    let state = SharedState::default();

    let mut broker = Broker::new((Ipv4Addr::UNSPECIFIED, 8080), state.clone()).await;

    let trace_layer = TraceLayer::new_for_http().make_span_with(|req: &Request<Body>| {
        tracing::info_span!(
            "request",
            method = req.method().as_str(),
            path = req.uri().path()
        )
    });

    let (router, openapi) =
        utoipa_axum::router::OpenApiRouter::<SharedState>::with_openapi(ApiDoc::openapi())
            .merge(api::router())
            .with_state(state)
            .split_for_parts();

    let router = router
        .merge(SwaggerUi::new("/docs").url("/docs/swagger.json", openapi.clone()))
        .layer(trace_layer)
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 8081)).await?;

    let web_task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_requested())
            .await
    });
    let broker_task = tokio::spawn(async move { broker.run().await });

    select! {
        _ = broker_task => {},
        _ = web_task => {},
        _ = shutdown_requested() => {
            info!("Received shutdown signal");
        },
    };
    Ok(())
}

async fn shutdown_requested() {
    // TODO: support linux signals
    if let Err(e) = tokio::signal::ctrl_c().await {
        error!("ctrl+c signal errored: {e}");
    };
}
