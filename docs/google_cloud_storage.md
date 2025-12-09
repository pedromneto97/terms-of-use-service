
# Google Cloud Storage Adapter — Unified Documentation

This documentation gathers all information about using the Google Cloud Storage (GCS) adapter in the Terms of Use service, including quick reference, examples, troubleshooting, and best practices.

---

### Environment Variables
| Variable                        | Description                              | Example                          |
|----------------------------------|------------------------------------------|----------------------------------|
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to the service account JSON file    | `/path/to/service-account.json`  |
| `GOOGLE_CLOUD_BUCKET`           | Bucket name (required)                   | `my-terms-bucket`                |

### Authentication Methods
- **Service Account JSON:** Recommended
- **Application Default Credentials (ADC):** Uses `gcloud auth application-default login` if no service account is set

### Supported Content Types
- `application/pdf` → `.pdf`
- `image/png` → `.png`
- `image/jpeg` → `.jpg`
- Others → detected automatically

### Example: Setting Environment Variables
```bash
export GOOGLE_CLOUD_BUCKET=my-bucket-name
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account-key.json # optional
```
