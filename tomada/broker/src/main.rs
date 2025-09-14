use std::net::Ipv4Addr;

use axum::{body::Body, http::Request};
use broker::{Broker, SharedState, api, cli::ARGS};
use tokio::{net::TcpListener, select};
use tower_http::trace::TraceLayer;
use tracing::{info, level_filters::LevelFilter};
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

    let mut broker = Broker::new((Ipv4Addr::UNSPECIFIED, ARGS.broker_port), state.clone()).await;

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
        .merge(SwaggerUi::new("/broker/docs").url("/broker/docs/swagger.json", openapi.clone()))
        .layer(trace_layer)
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ARGS.http_port)).await?;

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

#[cfg(unix)]
async fn shutdown_requested() {
    use tokio::signal::unix::SignalKind;

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())
        .expect("Failed to create a shutdown signal handler");
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("Failed to create a shutdown signal handler");

    futures::future::select(Box::pin(sigint.recv()), Box::pin(sigterm.recv())).await;
}

#[cfg(windows)]
async fn shutdown_requested() {
    use core::pin::Pin;

    let mut ctrl_c =
        tokio::signal::windows::ctrl_c().expect("Failed to create a ctrl+c signal handler");
    let mut ctrl_close =
        tokio::signal::windows::ctrl_close().expect("Failed to create a close signal handler");
    let mut ctrl_logoff =
        tokio::signal::windows::ctrl_logoff().expect("Failed to create a logoff signal handler");
    let mut ctrl_shutdown = tokio::signal::windows::ctrl_shutdown()
        .expect("Failed to create a shutdown signal handler");

    futures::future::select_all::<[Pin<Box<dyn Future<Output = _> + Send>>; 4]>([
        Box::pin(ctrl_c.recv()),
        Box::pin(ctrl_close.recv()),
        Box::pin(ctrl_logoff.recv()),
        Box::pin(ctrl_shutdown.recv()),
    ])
    .await;
}
