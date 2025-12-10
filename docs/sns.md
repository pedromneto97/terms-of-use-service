# SNS Adapter

AWS Simple Notification Service (SNS) adapter for publishing user agreement events. Enables event-driven architectures and real-time notifications when users accept terms of use.

## Environment Variables
| Variable          | Description                | Example                          |
|-------------------|----------------------------|----------------------------------|
| AWS_ACCESS_KEY_ID | AWS access key             | AKIAIOSFODNN7EXAMPLE             |
| AWS_SECRET_ACCESS_KEY | AWS secret key         | wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY |
| AWS_REGION        | AWS region                 | us-east-1                        |
| AWS_ACCOUNT_ID    | AWS account ID             | 123456789012                     |
| SNS_TOPIC_NAME    | SNS topic name             | terms-agreements                 |
| AWS_ENDPOINT_URL  | AWS Endpoint (optional)    | http://localhost:4566            |

## Quick Setup

### Environment Configuration
```bash
export AWS_REGION=us-east-1
export AWS_ACCOUNT_ID=123456789012
export SNS_TOPIC_NAME=terms-agreements
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
```

### LocalStack Development
```bash
# Use LocalStack for local testing
export AWS_ENDPOINT_URL=http://localhost:4566
```

## Message Format

Published messages contain user agreement details in JSON format:

```json
{
  "user_id": 123,
  "term_id": 456,
  "group": "privacy-policy"
}
```

## Use Cases

- **Event-Driven Processing**: Trigger downstream workflows when users accept terms
- **Analytics**: Feed agreement data to data warehouses or analytics pipelines
- **Compliance Logging**: Archive agreement events for audit trails
- **Notifications**: Send emails or notifications to administrators
- **Integrations**: Connect with other systems (CRM, billing, etc.)

## Feature Flag

Enable SNS publishing with the `sns` feature:

Build command:
```bash
cargo build --features "sns"
```

## Topic ARN Format

The topic ARN is automatically constructed as:
```
arn:aws:sns:{region}:{account_id}:{topic_name}
```

Ensure your AWS credentials have `sns:Publish` permissions for the specified topic.
