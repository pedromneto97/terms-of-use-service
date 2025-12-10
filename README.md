

# Terms of Use Service

A modular, production-ready Rust service for managing Terms of Use agreements. Built with a hexagonal architecture and feature flags, it allows you to **choose your own adapters** for database, cache, and storage at compile-time.

## ✨ Features
- **Modular hexagonal architecture** - Clean separation of concerns
- **Compile-time adapter selection** - No runtime overhead from unused features
- **Multiple API options** - Actix-web (HTTP) and Tonic (gRPC)
- **Flexible data layer** - Postgres or DynamoDB
- **Pluggable cache** - Redis or Valkey
- **Multi-cloud storage** - S3 or Google Cloud Storage
- **Event publishing** - AWS SNS for event-driven architectures
- **Full observability** - OpenTelemetry integration for tracing and logging

## 🚀 Quick Start

### Clone and Build
```bash
# Clone the repository
git clone https://github.com/pedromneto97/terms-of-use-service.git
cd terms-of-use-service

# Build with your chosen adapters
cargo build --release --features "actix-web,postgres,s3,otel"
```

### Configure Environment
```bash
# Database (Postgres example)
export DATABASE_URL=postgres://user:pass@localhost:5432/terms_db

# Cache (Redis example)
export REDIS_URL=redis://localhost:6379

# Storage (S3 example)
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export AWS_REGION=us-east-1
export S3_BUCKET=my-bucket

# Publisher (SNS example - optional)
export AWS_ACCOUNT_ID=123456789012
export SNS_TOPIC_NAME=terms-agreements

# API
export API_HOST=0.0.0.0
export API_PORT=8080
```

### Run the Service
```bash
cargo run --features "actix-web,postgres,s3,otel"
```

#### Example for AWS Stack
```bash
cargo run --features "actix-web,dynamodb,s3,sns,redis,otel"
```

The service will start on `http://0.0.0.0:8080`.

**👉 [Full Getting Started Guide →](GETTING_STARTED.md)**


## 📦 Adapter Guide

### Choose Your Stack
Before building, decide which adapters you need. Select **one from each required category**:

### Database Layer (Required)
| Adapter    | Status | Feature | Best For |
|:----------:|:------:|:-------:|----------|
| Postgres   | ✅     | `postgres` | Traditional relational databases, complex queries |
| DynamoDB   | ✅     | `dynamodb` | Serverless, auto-scaling, AWS-native |

### Cache Layer (Optional)
| Adapter    | Status | Feature | Best For |
|:----------:|:------:|:-------:|----------|
| Redis      | ✅     | `redis` | Fast in-memory caching, sessions |
| Valkey     | ❌     | `valkey` | Open-source Redis alternative |
| None       | ✅     | -       | No-op cache (default if no cache feature enabled) |

### API Layer (Required)
| Adapter     | Status | Feature      | Best For |
|:-----------:|:------:|:------------:|----------|
| Actix-web   | ✅     | `actix-web`  | HTTP REST APIs, traditional web apps |
| Tonic (gRPC)| ✅     | `grpc`      | High-performance gRPC services |

### Storage Layer (Required)
| Adapter     | Status | Feature   | Best For |
|:-----------:|:------:|:---------:|----------|
| S3          | ✅     | `s3`      | AWS object storage |
| Google      | ✅     | `gcloud`  | Google Cloud Storage |

### Publisher Layer (Optional)
| Adapter     | Status | Feature   | Best For |
|:-----------:|:------:|:---------:|----------|
| SNS         | ✅     | `sns`     | AWS event publishing, event-driven architectures |
| None        | ✅     | -         | No-op publisher (default if no publisher feature enabled) |

**Note:** Status shows currently available adapters. Cache and Publisher layers are optional and default to no-op implementations when not configured.

## 📖 Detailed Adapter Documentation

Each adapter has its own configuration guide with environment variables and setup instructions:

**Database:**
- [Postgres Setup](docs/postgres.md) - Traditional SQL database
- [DynamoDB Setup](docs/dynamodb.md) - AWS NoSQL database

**Cache:**
- [Redis Setup](docs/redis.md) - In-memory data store
- [Valkey Setup](docs/valkey.md) - Open-source Redis alternative

**Storage:**
- [S3 Setup](docs/s3.md) - AWS object storage
- [Google Cloud Storage Setup](docs/google_cloud_storage.md) - GCS buckets

**Publisher:**
- [SNS Setup](docs/sns.md) - AWS event publishing

**API:**
- [Actix-web Setup](docs/actix-web.md) - HTTP REST API
- [Tonic (gRPC) Setup](docs/grpc.md) - gRPC services

## 📡 Observability

This service includes comprehensive observability through Open Telemetry integration. See [Open Telemetry Configuration](docs/open-telemetry.md) for details on enabling and configuring telemetry for tracing, metrics, and logging.

## 🏗️ Architecture
See `.github/copilot-instructions.md` for a full architecture overview. Layer-specific instructions are in `.github/instructions/`.

## 🤝 Contributing
PRs and issues welcome! Please follow Rust best practices and maintain the modular hexagonal design. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License
See [LICENSE](LICENSE) for details.
