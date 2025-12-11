---
applyTo: domain/**
---
# Domain Layer Instructions

## Purpose
Hold all business logic, entities, DTOs, and traits. Must stay free of infrastructure (no HTTP, DB, or SDK imports).

## Structure
```
domain/
├── entities.rs      # Domain models
├── dto.rs           # DTOs used by use cases
├── errors.rs        # TermsOfUseError and variants
├── data/
│   ├── repository.rs  # Traits: TermRepository, UserAgreementRepository
│   └── service/       # Traits: CacheService, StorageService, PublisherService
└── use_cases/         # Business rules orchestrated by use cases
```

## Conventions
- Use cases accept `&impl Trait` and return `Result<_, TermsOfUseError>`; no concrete infra types.
- Keep validation and business rules here; inbound/outbound should not re-validate business rules.
- Map all failures to `TermsOfUseError`; avoid leaking infra specifics.
- Provide unit tests with mocked traits (`mockall`) covering success and validation errors.

## Core traits
- `TermRepository`, `UserAgreementRepository` in `data/repository.rs`.
- `CacheService`, `StorageService`, `PublisherService` in `data/service/` (noop implementations exist in outbound when features are off).

## Adding a use case
1) Add a file in `domain/use_cases/` and export it from `use_cases/mod.rs`.
2) Accept dependencies as trait references; do not depend on concrete types or config.
3) Enforce business validation inside the use case; return meaningful `TermsOfUseError` variants.
4) Add unit tests for happy path and failure scenarios.