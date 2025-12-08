# Postgres Adapter

A reliable, open-source relational database adapter for the Terms of Use service. Perfect for complex queries and traditional ACID transactions.
Uses SeaORM for database interactions.

## Environment Variables
| Variable      | Description         | Example                                    |
|---------------|--------------------|--------------------------------------------|
| DATABASE_URL  | Connection string  | postgres://user:pass@host:port/dbname      |

## Quick Setup

### Environment Configuration
```bash
export DATABASE_URL=postgres://terms_user:secret@localhost:5432/terms_db
```
