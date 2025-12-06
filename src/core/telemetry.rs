use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::{BatchLogProcessor, SdkLoggerProvider};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider, trace::SdkTracerProvider};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct OtelSdk {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: SdkLoggerProvider,
}

impl Drop for OtelSdk {
    fn drop(&mut self) {
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("Failed to shutdown tracer provider: {:?}", e);
        }
        if let Err(e) = self.meter_provider.shutdown() {
            eprintln!("Failed to shutdown meter provider: {:?}", e);
        }
        if let Err(e) = self.logger_provider.shutdown() {
            eprintln!("Failed to shutdown log provider: {:?}", e);
        }
    }
}

fn setup_logger(resource: Resource, otel_endpoint: &str) -> SdkLoggerProvider {
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()
        .expect("Failed to create OTLP log exporter");

    SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_log_processor(BatchLogProcessor::builder(log_exporter).build())
        .build()
}

fn setup_tracer(
    resource: Resource,
    otel_endpoint: &str,
    logger_provider: &SdkLoggerProvider,
) -> SdkTracerProvider {
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()
        .expect("Failed to create OTLP span exporter");

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(span_exporter)
        .build();

    let tracer = tracer_provider.tracer("terms-of-use");

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let otel_logs_layer = OpenTelemetryTracingBridge::new(logger_provider);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(telemetry_layer)
        .with(otel_logs_layer)
        .init();

    tracer_provider
}

fn setup_meter(resource: Resource, otel_endpoint: &str) -> SdkMeterProvider {
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()
        .expect("Failed to create OTLP metric exporter");

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_periodic_exporter(metric_exporter)
        .build();

    global::set_meter_provider(meter_provider.clone());
    meter_provider
}

pub fn init_telemetry() -> OtelSdk {
    let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "terms-of-use".to_string());

    let resource = Resource::builder()
        .with_service_name(service_name.clone())
        .build();

    let logger_provider = setup_logger(resource.clone(), &otel_endpoint);
    let tracer_provider = setup_tracer(resource.clone(), &otel_endpoint, &logger_provider);
    let meter_provider = setup_meter(resource, &otel_endpoint);

    OtelSdk {
        tracer_provider,
        meter_provider,
        logger_provider,
    }
}
