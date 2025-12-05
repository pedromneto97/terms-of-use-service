---
applyTo: src/inbound/**
---
# Inbound Layer Instructions

## Purpose
Exposes external interfaces (REST, gRPC) to clients. Delegates all business logic to Domain layer.

## Available Implementations
| Feature      | Framework   | Directory               |
|--------------|-------------|-------------------------|
| `actix-web`  | Actix Web 4 | `src/inbound/actix/`    |
| `tonic`      | Tonic gRPC  | `src/inbound/tonic/`    |

## Conventions
- **Validate input** at the API boundary before calling use cases
- **Return standardized responses**: status, message, payload structure
- **Error handling**: Map domain errors to appropriate HTTP/gRPC status codes (see `error/` module)
- **Versioning**: Endpoints organized in `v1/`, `v2/` directories

## Adding a New Endpoint
1. Create handler function in appropriate version directory (`v1/`, etc.)
2. Wire route in module's `configure()` function
3. Call domain use case, passing required services from `Config`
4. Map result to API response format

## Example: Actix Handler Pattern
```rust
pub async fn create_term(
    config: web::Data<Config>,
    payload: web::Json<CreateTermRequest>,
) -> impl Responder {
    let result = create_term_of_use_use_case(
        &config.repository,
        &config.storage,
        &config.cache,
        payload.into_inner().into(),
        &file_path,
    ).await;
    
    match result {
        Ok(term) => HttpResponse::Ok().json(term),
        Err(e) => e.into(), // Uses error module conversion
    }
}
```

## Key Files
- `src/inbound/mod.rs` — Feature-gated module inclusion and `start_server()` entry point
- `src/inbound/actix/v1/` — REST endpoint handlers
- `src/inbound/actix/error/` — Error-to-response mappings
