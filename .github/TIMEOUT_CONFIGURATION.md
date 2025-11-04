# CI/CD Timeout Configuration

This document outlines the timeout configuration for all GitHub Actions workflows.

## Timeout Strategy

### Job-Level Timeouts

All jobs have timeout limits based on their expected duration:

*   **rust.yml**: `test` (30m), `clippy` (20m)
*   **migrations.yml**: `migration-syntax-check` (10m), `migration-check` (15m)
*   **deploy.yml**: `deployment-check` (5m), `deploy` (25m), `notify` (2m)

### Step-Level Timeouts

Critical operations have individual step timeouts:

*   **Build workspace**: 20 minutes
*   **Security audit**: 5 minutes
*   **Database connectivity**: 8 minutes
*   **Health checks**: 15 minutes

### Command-Level Timeouts

Individual commands have their own timeout limits:

*   `curl` operations: 10-30 seconds
*   `cargo bench`: 10 minutes
*   SurrealDB connectivity: 10-30 seconds per attempt

### Retry Logic Limits

Network operations use controlled retry mechanisms:

*   **Database Connectivity**: 5 attempts, 10-30s timeout, 60s max wait, 8m total limit.
*   **Health Checks**: 12 attempts, 10s timeout, 45s max wait, 15m total limit.

## Fail-Fast Configuration

*   Build failures immediately stop dependent jobs.
*   Security audit failures block deployment.
*   Database connectivity failures skip migration application.

## Cost Optimization

### Expected CI Minutes Per Workflow Run

*   **Pull Request**: ~20-35 minutes
*   **Main Branch Push**: ~40-65 minutes

### Cost Control Measures

1.  Aggressive timeout limits.
2.  Reduced retry counts.
3.  Capped wait times.
4.  Selective execution.

## Troubleshooting

### Common Timeout Issues

*   **"Job was cancelled due to timeout"**: Check if the timeout is appropriate. Look for hanging processes or network issues.
*   **"Step timed out after X minutes"**: Identify the slow operation. Check network connectivity and build cache effectiveness.
*   **"Health checks failed repeatedly"**: Verify the application is deployed and running. Check DNS configuration and application startup time.

## Maintenance

This timeout configuration should be reviewed quarterly to:

1.  Adjust limits based on actual usage.
2.  Optimize for new tools or dependencies.
3.  Account for infrastructure improvements.

Last updated: 2025-09-06