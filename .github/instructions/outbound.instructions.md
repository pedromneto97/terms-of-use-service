---
applyTo: src/outbound/**
---
# Outbound Layer Instructions

## Purpose
Implements infrastructure adapters (database, storage, cache) that fulfill domain traits. Selected at compile-time via Cargo features.

## Available Implementations
| Category | Feature    | Directory                      | Implements              |
|----------|------------|--------------------------------|-------------------------|
| Database | `postgres` | `src/outbound/database/postgres/` | `TermRepository`, `UserAgreementRepository` |
| Database | `dynamodb` | `src/outbound/database/dynamodb/` | `TermRepository`, `UserAgreementRepository` |
| Storage  | `s3`       | `src/outbound/storage/s3/`     | `UploadService`         |
| Storage  | `google`   | `src/outbound/storage/google/` | `UploadService`         |
| Cache    | `redis`    | (planned)                      | `CacheService`          |
| Cache    | `valkey`   | (planned)                      | `CacheService`          |

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
```

## Conventions
- **Implement domain traits exactly** — no extra public methods on adapters
- **Handle infrastructure errors** by mapping to `TermsOfUseError`
- **Use SeaORM** for SQL databases (`postgres` feature)
- **Use AWS SDK** for S3/DynamoDB (`aws-sdk-s3`, `aws-config` dependencies)

## Configuration Wiring (`src/core/config/`)
`Config` struct selects implementations based on features:
- `repository.rs` — Database connection and repository instance
- `storage.rs` — Storage client configuration
- `cache.rs` — Cache client (defaults to `NoopCacheService` if no cache feature)

## Migrations (`migration/`)
For SQL databases, use SeaORM migrations:
- Generate: `cd migration && cargo run -- generate MIGRATION_NAME`
- Apply: `cargo run -- up`
- Rollback: `cargo run -- down`
- Status: `cargo run -- status`

## Key Files
- `src/outbound/database/postgres/` — Postgres implementation example
- `src/outbound/storage/s3/` — S3 implementation example
- `src/core/config/mod.rs` — Feature-based wiring
