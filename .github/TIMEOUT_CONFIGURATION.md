# GitHub Actions Timeout Configuration

This document explains the timeout configurations for GitHub Actions workflows. Timeouts are set at the job and step level to prevent runaway builds, manage costs, and identify stalled processes.

## Why Timeouts Are Essential

1.  **Cost Management**: GitHub Actions incur costs per minute. Timeouts prevent excessive billing from jobs that get stuck or become inefficient.
2.  **Maintain Performance**: Fail-fast timeouts ensure that long-running or stalled jobs do not bottleneck the build queue, allowing subsequent workflows to proceed.
3.  **Early Error Detection**: A timeout indicates a broken process, an inefficient operation, or a network issue, prompting quicker investigation.

## Timeout Settings Overview

Timeouts are configured based on the expected run time of each job, with an added buffer.

### Main Workflow (`rust.yml`)

-   **`test` job (30 minutes)**: This job executes the full integration test suite, which is the longest part of the build. A 30-minute timeout provides a safe buffer for the typical 15-minute test run.
-   **`clippy` job (20 minutes)**: The linter can be slow with a cold cache. This timeout prevents it from running indefinitely.

### Deployment Workflow (`deploy.yml`)

-   **`deploy` job (25 minutes)**: Deployment involves building and pushing a container, then waiting for service health checks. This timeout accounts for potential network latency and variable deployment times.

### Step-Level Timeouts

Timeouts are also applied to specific steps within jobs, particularly for processes involving network calls or potentially long durations.

-   **Health Checks (15 minutes total)**: After deployment, a script repeatedly checks the application's health endpoint. The script runs for up to 15 minutes (12 attempts with increasing delays) before reporting a failure.
-   **Database Connectivity (8 minutes total)**: The `ensure-db-ready.sh` script attempts to connect to the database up to 5 times. The entire process has an 8-minute cap before it fails.

## Troubleshooting Timeouts

-   **"Job cancelled due to timeout"**: If a job is cancelled by a timeout, review the logs to pinpoint the step that was executing. This often points to a network problem or a hung process.
-   **"Health checks failed"**: This usually means the application did not start correctly after deployment. Examine the application's runtime logs in the hosting environment (e.g., DigitalOcean App Platform) to diagnose the startup error.

These timeout settings should be reviewed periodically as the project evolves.

---
*Last Updated: 2025-11-06*