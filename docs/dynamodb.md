# DynamoDB Adapter

AWS DynamoDB adapter for serverless, scalable database storage. Perfect for cloud-native deployments on AWS with automatic scaling and high availability.

## Environment Variables
| Variable               | Description         | Example         |
|------------------------|--------------------|-----------------|
| AWS_ACCESS_KEY_ID      | AWS access key     | AKIAIOSFODNN7EXAMPLE |
| AWS_SECRET_ACCESS_KEY  | AWS secret key     | wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY |
| AWS_REGION             | AWS region         | us-east-1       |
| AWS_ENDPOINT_URL       | AWS Enpoint        | http://localhost:4566 |


### Environment Configuration
```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1
```


### Build and Run
```bash
cargo build --features "dynamodb"
cargo run --features "dynamodb"
```
