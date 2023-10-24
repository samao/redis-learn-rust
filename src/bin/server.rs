/*
 * Copyright (c) QieTv, Inc. 2018
 * @Author: idzeir
 * @Date: 2023-10-23 14:10:21
 * @Last Modified by: idzeir
 * @Last Modified time: 2023-10-23 14:35:36
 */
use clap::Parser;
use mini_redis::{server, DEFAULT_PORT};
use tokio::{net::TcpListener, signal};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "otel")]
// To be able to set the XrayPropagator
use opentelemetry::global;
#[cfg(feature = "otel")]
// To configure certain options such as sampling rate
use opentelemetry::sdk::trace as sdktrace;
#[cfg(feature = "otel")]
// For passing along the same XrayId across services
use opentelemetry_aws::trace::XrayPropagator;
#[cfg(feature = "otel")]
// The `Ext` traits are to allow the Registry to accept the
// OpenTelemetry-specific types (such as `OpenTelemetryLayer`)
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, util::TryInitError, EnvFilter,
};

#[tokio::main]
pub async fn main() -> mini_redis::Result<()> {
    set_up_logging()?;
    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;
    server::run(listener, signal::ctrl_c()).await;
    Ok(())
}

#[derive(Debug, Parser)]
#[clap(name = "mini-redis-server", version, author, about = "A Redis server")]
struct Cli {
    #[clap(long)]
    port: Option<u16>,
}

#[cfg(not(feature = "otel"))]
fn set_up_logging() -> mini_redis::Result<()> {
    tracing_subscriber::registry().with(fmt::layer()).init();
    // tracing_subscriber::fmt::try_init()?;
    Ok(())
}

#[cfg(feature = "otel")]
fn set_up_logging() -> Result<(), TryInitError> {
    use tracing_subscriber::{fmt, EnvFilter};

    opentelemetry::global::set_text_map_propagator(XrayPropagator::default());
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            sdktrace::config()
                .with_sample(sdktrace::Sampler::AlwaysOn)
                .with_id_generator(sdktrace::XrayPropagator::default()),
        )
        .install_simple()
        .expect("unable to initialize OtlpPipeline");
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(filter)
        .with(fmt::Layer::default())
        .try_init()
}
