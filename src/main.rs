use std::{env::args, net::SocketAddr};

use axum_server::tls_rustls::RustlsConfig;
use dotenvy::{dotenv, var};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{Level, info};
use tracing_subscriber::{filter::Targets, prelude::*};

use packhub::{app, cron::run_cron_job, state::AppState};

#[tokio::main]
async fn main() {
    let filter = Targets::new()
        .with_target("tower_http", Level::TRACE)
        .with_target("packhub", Level::TRACE)
        .with_default(Level::INFO);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    if dotenv().is_err() {
        info!("No .env file found");
    }

    let mut generate_keys = false;

    if args().len() > 1 {
        let arg = args().nth(1).unwrap();

        if arg == "--generate-keys" {
            generate_keys = true;
        }
    }

    let state = AppState::initialize(generate_keys).await;

    // Run the cron job to update platform detection data at startup
    run_cron_job(state.clone()).await;

    let cron = JobScheduler::new().await.unwrap();

    let state2 = state.clone();

    cron.add(
        Job::new_async("0 */4 * * * *", move |_uuid, _l| {
            let state2 = state2.clone();
            Box::pin(async move {
                info!("Running cron job: Updating platform detection data");
                run_cron_job(state2).await;
                info!("Finished cron job: Updating platform detection data");
            })
        })
        .unwrap(),
    )
    .await
    .unwrap();

    cron.start().await.unwrap();

    let http_addr: SocketAddr = format!("0.0.0.0:{}", var("PACKHUB_HTTP_PORT").unwrap())
        .parse()
        .unwrap();

    let https_addr: SocketAddr = format!("0.0.0.0:{}", var("PACKHUB_HTTPS_PORT").unwrap())
        .parse()
        .unwrap();

    info!("listening on {}", http_addr);
    info!("listening on {}", https_addr);

    let config = RustlsConfig::from_pem_file(
        var("PACKHUB_CERT_PEM").unwrap(),
        var("PACKHUB_KEY_PEM").unwrap(),
    )
    .await
    .unwrap();

    let http_server = axum_server::bind(http_addr).serve(app(state.clone()).into_make_service());

    let https_server =
        axum_server::bind_rustls(https_addr, config).serve(app(state).into_make_service());

    let http = tokio::spawn(async { http_server.await.unwrap() });
    let https = tokio::spawn(async { https_server.await.unwrap() });

    _ = tokio::join!(http, https);
}
