use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing::info;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use crate::config::LogConfig;

pub fn init_logger(config: &LogConfig) {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.otel_endpoint),
        )
        .with_trace_config(
            sdktrace::Config::default().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                config.service_name.clone(),
            )])),
        )
        .install_simple()
        .expect("failed to init otel");

    let otel_layer = OpenTelemetryLayer::new(tracer);

    let filter = EnvFilter::new(&config.level);

    Registry::default()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .with(otel_layer)
        .try_init()
        .expect("failed to init tracing");

    let meter = global::meter("transaction-service");

    let startup_counter: Counter<u64> = meter
        .u64_counter("service_startups_total")
        .with_description("Number of times the service has started")
        .init();

    startup_counter.add(1, &[KeyValue::new("service", config.service_name.clone())]);

    info!("logger + otel initialized");
}

pub fn shutdown_tracer() {
    let meter = global::meter("transaction-service");
    let shutdown_counter: Counter<u64> = meter
        .u64_counter("service_shutdowns_total")
        .with_description("Number of graceful shutdowns")
        .init();
    shutdown_counter.add(1, &[KeyValue::new("service", "transaction-service")]);
    global::shutdown_tracer_provider();
}
