# Open Telemetry Configuration

This service integrates Open Telemetry for comprehensive observability including tracing, metrics, and logging. Understand what's happening in your service with full visibility.

## Environment Variables
| Variable             | Description                | Default | Example        |
|----------------------|----------------------------|---------|-----------------|
| RUST_LOG             | Log level                  | info    | debug, warn     |
| OTEL_EXPORTER_OTLP_ENDPOINT | OTLP collector endpoint | -       | http://localhost:4317 |
| OTEL_SERVICE_NAME    | Service name for traces    | terms-of-use | my-service |


