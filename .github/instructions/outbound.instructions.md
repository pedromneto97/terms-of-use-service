---
applyTo: src/outbound/**
---
# Outbound Layer Instructions

## Purpose
Implements infrastructure adapters (database, storage, cache) that fulfill domain traits. Selected at compile-time via Cargo features.

## Available Implementations
| Category  | Feature    | Directory                         | Implements                                      | Dependencies                    |
|-----------|------------|-----------------------------------|------------------------------------------------|----------------------------------|
| Database  | `postgres` | `src/outbound/database/postgres/` | `TermRepository`, `UserAgreementRepository`     | `sea-orm`, `migration`          |
| Database  | `dynamodb` | `src/outbound/database/dynamodb/` | `TermRepository`, `UserAgreementRepository`     | `aws-sdk-dynamodb`, `aws-config`|
| Storage   | `s3`       | `src/outbound/storage/s3/`        | `StorageService`                                | `aws-sdk-s3`, `aws-config`      |
| Storage   | `gcloud`   | `src/outbound/storage/gcloud/`    | `StorageService`                                | `google-cloud-storage`          |
| Cache     | `redis`    | `src/outbound/cache/redis/`       | `CacheService`                                  | `deadpool-redis`, `serde_json`  |
| Cache     | none       | `src/outbound/cache/noop/`        | `CacheService` (no-op default)                  | none                            |
| Publisher | `sns`      | `src/outbound/publisher/sns/`     | `PublisherService`                              | `aws-sdk-sns`, `aws-config`     |
| Publisher | none       | `src/outbound/publisher/noop/`    | `PublisherService` (no-op default)              | none                            |

## Adding a New Adapter
1. Create directory under appropriate category: `src/outbound/{database|storage|cache}/new_adapter/`
2. Implement the required domain trait(s)
3. Add feature flag in `Cargo.toml`:
   ```toml
   [features]
   new_adapter = ["dep:new-adapter-crate"]
   ```
4. Gate module with `#[cfg(feature = "new_adapter")]` in parent `mod.rs`
5. Wire in `src/core/config/` to select based on feature

## Module Structure Pattern
```rust
// src/outbound/database/mod.rs
#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "dynamodb")]
pub mod dynamodb;

// src/outbound/cache/mod.rs
pub mod noop;  // Always available as default

#[cfg(feature = "redis")]
pub mod redis;

// src/outbound/publisher/mod.rs
pub mod noop;  // Always available as default

#[cfg(feature = "sns")]
pub mod sns;
```

## Conventions
- **Implement domain traits exactly** — no extra public methods on adapters
- **Handle infrastructure errors** by mapping to `TermsOfUseError`
- **Use SeaORM** for SQL databases (`postgres` feature)
- **Use AWS SDK** for AWS services (`aws-sdk-*` with `aws-config`)
- **Use Google Cloud SDK** for GCP services (`google-cloud-*`)
- **Provide no-op implementations** for optional services (cache, publisher) as defaults
- **Use feature flags** to enable specific implementations at compile-time

## Configuration Wiring (`src/core/config/`)
`Config` struct selects implementations based on features:
- `repository.rs` — Database connection and repository instances (TermRepository, UserAgreementRepository)
- `storage.rs` — Storage client configuration (S3, Google Cloud Storage)
- `cache.rs` — Cache client (Redis or NoopCacheService as default)
- `publisher.rs` — Publisher client (SNS or NoopPublisherService as default)

## Migrations (`migration/`)
For SQL databases, use SeaORM migrations:
- Generate: `cd migration && cargo run -- generate MIGRATION_NAME`
- Apply: `cargo run -- up`
- Rollback: `cargo run -- down`
- Status: `cargo run -- status`

## Key Files
- `src/outbound/database/postgres/` — PostgreSQL implementation using SeaORM
- `src/outbound/database/dynamodb/` — DynamoDB implementation using AWS SDK
- `src/outbound/storage/s3/` — AWS S3 implementation
- `src/outbound/storage/gcloud/` — Google Cloud Storage implementation
- `src/outbound/cache/redis/` — Redis cache implementation
- `src/outbound/cache/noop/` — No-op cache (default)
- `src/outbound/publisher/sns/` — AWS SNS publisher implementation
- `src/outbound/publisher/noop/` — No-op publisher (default)
- `src/core/config/mod.rs` — Feature-based wiring and configuration
