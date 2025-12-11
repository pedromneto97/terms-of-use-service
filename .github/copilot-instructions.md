# Copilot Instructions for AI Agents

## Essential context
- Rust service for Terms of Use management using a hexagonal layout: `domain/` (business + traits), `inbound/` (Actix HTTP, gRPC), `outbound/` (DB, storage, cache, publisher), `core/config/` (feature wiring).
- Everything talks to the domain via traits; no infrastructure leaks into domain.

## Feature flags (root Cargo.toml)
- Inbound: `actix-web`, `grpc`
- Database: `postgres`, `dynamodb`
- Storage: `s3`, `gcloud`
- Cache: `redis` (noop default)
- Publisher: `sns`, `kafka` (noop default)
- Telemetry: `otel`
Example: `cargo build --features "actix-web,postgres,s3"`

## Workflows
- Build: `cargo build --features "<features>"`
- Run: `cargo run --features "<features>"`
- Test: `cargo test`
- Migrations (SQL): `cargo run -- up|down|status` in `migration/`
- Local AWS: `./localstack/setup-aws.sh`

## Where to find detailed rules
- Inbound: `.github/instructions/inbound.instructions.md`
- Domain: `.github/instructions/domain.instructions.md`
- Outbound: `.github/instructions/outbound.instructions.md`

## Telemetry
- Enable with feature `otel`; configuration lives in `src/core/telemetry.rs`.
