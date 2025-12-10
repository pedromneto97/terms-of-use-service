---
applyTo: src/domain/**
---
# Domain Layer Instructions

## Purpose
Contains all business logic, entities, and trait definitions. Must remain independent of infrastructure (no direct database or HTTP dependencies).

## Structure
```
src/domain/
├── entities.rs      # Core domain models (TermOfUse, etc.)
├── dto.rs           # Data transfer objects for use cases
├── errors.rs        # Domain error types
├── data/
│   ├── mod.rs
│   ├── repository.rs  # Repository traits (TermRepository, UserAgreementRepository)
│   └── service/       # Service traits (CacheService, StorageService, PublisherService)
│       ├── mod.rs
│       ├── cache.rs
│       ├── storage.rs
│       └── publisher.rs
└── use_cases/         # Business logic orchestration
```

## Core Traits
Outbound adapters must implement these traits:

### Repository Traits (`data/repository.rs`)
- `TermRepository`: `get_latest_term_for_group`, `get_term_by_id`, `create_term`
- `UserAgreementRepository`: `has_user_agreed_to_term`, `create_user_agreement`

### Service Traits (`data/service/`)
- `CacheService` (`cache.rs`): Cache operations for terms and agreements
  - Methods: `find_user_agreement`, `store_user_agreement`, `get_latest_term_for_group`, `store_latest_term_for_group`, `invalidate_cache_for_group`
  - Default implementation: `NoopCacheService` when no cache feature is enabled
- `StorageService` (`storage.rs`): File upload/delete/URL operations
  - Methods: `upload_file`, `delete_file`, `get_file_url`
- `PublisherService` (`publisher.rs`): Event publishing for user agreements
  - Methods: `publish_agreement`
  - Default implementation: `NoopPublisherService` when no publisher feature is enabled

## Adding a New Use Case
1. Create file in `src/domain/use_cases/` (e.g., `revoke_agreement.rs`)
2. Define async function accepting trait references (not concrete types):
   ```rust
   pub async fn my_use_case(
       repository: &impl TermRepository,
       cache: &impl CacheService,
       // ...params
   ) -> Result<ReturnType, TermsOfUseError>
   ```
3. Export in `use_cases/mod.rs`
4. Wire to API endpoint in inbound layer

## Conventions
- **Never import infrastructure code** — only use traits
- **All validation logic** belongs here, not in API or Data layers
- **Use `TermsOfUseError`** for all error handling
- **Testable**: Use mock trait implementations for unit tests

## Key Files
- `src/domain/use_cases/create_term_of_use.rs` — Example of complete use case pattern
- `src/domain/data/repository.rs` — Repository trait definitions
- `src/domain/data/service/cache.rs` — Cache trait and NoopCacheService
