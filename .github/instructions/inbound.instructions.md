---
applyTo: inbound/**
---
# Inbound Layer Instructions

## Purpose
Expose HTTP (Actix) and gRPC (Tonic) endpoints that delegate all business logic to the Domain layer.

## Features and layout
- `actix-web`: REST handlers under `inbound/src/actix/` with routes grouped by version (e.g., `v1/`).
- `grpc`: Tonic server under `inbound/src/grpc/` (includes upload helpers and mappers).

## Conventions
- Validate inputs at the boundary; reject unsupported content types (PDF only for term uploads).
- Map errors to `ProblemDetails` (Actix) or `tonic::Status` (gRPC); do not leak internal details.
- Use mappers/helpers for DTO <-> domain conversions; keep handlers thin.
- Add `#[tracing::instrument]` to handlers; avoid logging sensitive data.
- Keep streaming uploads in temp files, then call `create_term_of_use_use_case`.

## Adding an endpoint
1) Add handler in the right versioned module. 2) Wire routing (`configure` for Actix or service registration for gRPC). 3) Call the appropriate use case, passing trait-based services from `Config`. 4) Map result to the API response shape.

## Key files
- `inbound/src/lib.rs` — Feature-gated module inclusion and server startup.
- `inbound/src/actix/` — REST handlers, payloads, and error mapping.
- `inbound/src/grpc/` — Tonic server, mappers, and file upload helpers.
- `inbound/src/config.rs` — Configuration struct with trait-based service dependencies.
