# Copilot Instructions for AI Agents

## Project Overview
Rust project for managing Terms of Use agreements. Fully modular architecture with feature flags for compile-time selection of implementations.

## Architecture (Hexagonal)
```
src/
├── core/config/     # Configuration for repository, cache, and storage
├── domain/          # Business logic, entities, use cases, traits
├── inbound/         # API adapters (actix, tonic)
└── outbound/        # Infrastructure adapters (database, storage, cache)
```

## Feature Flags (Cargo.toml)
Compile with only the features you need:
| Layer    | Features                  |
|----------|---------------------------|
| Inbound  | `actix-web`, `tonic`      |
| Cache    | `redis`, `valkey`         |
| Storage  | `s3`, `google`            |
| Database | `postgres`, `dynamodb`    |

Example: `cargo build --features "actix-web,postgres,s3"`

## Developer Workflows
- **Build**: `cargo build --features "<features>"`
- **Run**: `cargo run --features "<features>"`
- **Test**: `cargo test`
- **Migrations** (`migration/`): `cargo run -- up`, `cargo run -- down`, `cargo run -- status`
- **Local AWS**: `./localstack/setup-aws.sh`

## Key Patterns
- **Trait-based abstractions**: All outbound adapters implement domain traits (`TermRepository`, `CacheService`, `UploadService`)
- **Use cases orchestrate logic**: See `src/domain/use_cases/` for business flows
- **Config wiring**: `src/core/config/` selects implementations based on features

## Layer-Specific Instructions
See detailed instructions for each layer:
- `.github/instructions/inbound.instructions.md` — API layer (Actix, Tonic)
- `.github/instructions/domain.instructions.md` — Business logic and use cases
- `.github/instructions/outbound.instructions.md` — Database, storage, cache adapters
