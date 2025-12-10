#!/bin/bash

# Create test bucket
aws --endpoint-url=http://localhost:4566 s3 mb s3://terms-of-use-bucket

# Create test topic
aws --endpoint-url=http://localhost:4566 sns create-topic --name terms-of-use-topic