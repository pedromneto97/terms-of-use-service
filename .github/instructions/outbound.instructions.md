---
applyTo: outbound/**
---
# Outbound Layer Instructions

## Purpose
Implement infrastructure adapters (DB, storage, cache, publisher) that satisfy domain traits. Selection is compile-time via Cargo features.

## Features and directories
| Category  | Feature    | Directory                         | Implements                                      |
|-----------|------------|-----------------------------------|-------------------------------------------------|
| Database  | `postgres` | `outbound/src/database/postgres/` | `TermRepository`, `UserAgreementRepository`     |
| Database  | `dynamodb` | `outbound/src/database/dynamodb/` | `TermRepository`, `UserAgreementRepository`     |
| Storage   | `s3`       | `outbound/src/storage/s3/`        | `StorageService`                                |
| Storage   | `gcloud`   | `outbound/src/storage/gcloud/`    | `StorageService`                                |
| Cache     | `redis`    | `outbound/src/cache/redis/`       | `CacheService`                                  |
| Cache     | default    | `outbound/src/cache/noop/`        | `CacheService` (no-op)                          |
| Publisher | `sns`      | `outbound/src/publisher/sns/`     | `PublisherService`                              |
| Publisher | `kafka`    | `outbound/src/publisher/kafka/`   | `PublisherService`                              |
| Publisher | default    | `outbound/src/publisher/noop/`    | `PublisherService` (no-op)                      |

## Conventions
- Implement only the domain trait methods; keep adapter APIs minimal.
- Map SDK/DB errors to `TermsOfUseError`; log context without sensitive data.
- Guard modules with `#[cfg(feature = "...")]`; provide noop defaults for cache/publisher.
- Use `#[tracing::instrument]` on external calls of interest.
- Keep constructors simple (`new`, `from_env`); wiring lives in `src/core/config/`.
- For SQL, use SeaORM; migrations live in `migration/` crate.

## Adding an adapter
1) Create a module under the right category and implement the required domain traits.
2) Add a feature flag and dependencies in `Cargo.toml` (root and `outbound/` as needed).
3) Gate the module in the parent `mod.rs`.
4) Wire selection in `src/core/config/`.

## Configuration wiring
- `core/config/repository.rs` — DB clients and repositories.
- `core/config/storage.rs` — Storage clients (S3 or GCS).
- `core/config/publisher.rs` — Publishers (SNS, Kafka, or noop).
- `outbound/src/cache/mod.rs` — Redis or noop cache selection.
