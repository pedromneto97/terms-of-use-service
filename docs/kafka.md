# Kafka Publisher

This document describes the Kafka publisher implementation for publishing term acceptance events.

## Overview

The Kafka publisher sends term acceptance events to a Kafka topic. It uses the `rdkafka` crate for Kafka integration.

## Configuration

The Kafka publisher is configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `KAFKA_BROKERS` | Comma-separated list of Kafka broker addresses | `localhost:9092` |
| `KAFKA_TOPIC` | Kafka topic name for term acceptance events | `terms-of-use-agreements` |

## Feature Flag

Enable the Kafka publisher by adding the `kafka` feature flag:

```bash
cargo build --features "kafka,actix-web,postgres,s3"
```

**Note:** Only one publisher can be enabled at a time. Do not enable both `kafka` and `sns` features simultaneously. If you need to switch publishers, rebuild with the appropriate feature flag.

## Message Format

Messages are published with the following structure:

**Key**: `{group}:{term_id}:{user_id}`

The key is organized hierarchically from broadest to most specific scope:
- **group**: The application/context where the term belongs (e.g., "mobile-app", "web-app", "terms-of-service")
- **term_id**: The specific term of use within that group
- **user_id**: The individual user accepting the term

This key structure ensures:
1. All terms for the same group are in the same partition (useful for listeners interested in one application's events)
2. Within a group, all versions of the same term stay together
3. Consumers can easily filter by group, term, or track individual users

**Value**: JSON representation of `AcceptedTermOfUseDTO`:
```json
{
  "user_id": 123,
  "term_id": 456,
  "group": "terms-of-service"
}
```

## Producer Configuration

The Kafka producer is configured with the following settings:

- `bootstrap.servers`: Connection string for Kafka brokers
- `message.timeout.ms`: 5000ms timeout for message delivery
- `queue.buffering.max.messages`: Maximum 10,000 messages in queue
- `queue.buffering.max.kbytes`: Maximum 1GB of buffered messages
- `batch.num.messages`: Batches of 100 messages

## Environment Setup

```bash
export KAFKA_BROKERS=localhost:9092
export KAFKA_TOPIC=terms-of-use-agreements
```

## Error Handling

The publisher handles the following error scenarios:

1. **Serialization errors**: If the DTO cannot be serialized to JSON, returns `TermsOfUseError::InternalServerError`
2. **Publishing errors**: If Kafka fails to accept the message, returns `TermsOfUseError::InternalServerError`

All errors are logged with appropriate context using the `tracing` framework.

## Observability

The `publish_agreement` method is instrumented with tracing. Each publication includes:

- User ID
- Term ID
- Group name
- Success/failure status

Traces are automatically captured when OpenTelemetry is enabled.
