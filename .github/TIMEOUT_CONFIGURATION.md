# GitHub Actions Timeout Configuration

This document outlines the timeout configuration for the GitHub Actions workflows. Timeouts are configured at multiple levels to prevent runaway builds, control costs, and detect hanging processes.

## Timeout Rationale

1.  **Cost Management:** GitHub Actions are billed by the minute. Timeouts prevent excessive costs from stuck or inefficient jobs.
2.  **Performance:** Fail-fast timeouts ensure that long-running or stalled jobs do not block the build queue for other workflows.
3.  **Error Detection:** A timeout serves as a clear signal that a process is broken, inefficient, or has encountered a network issue.

## Timeout Settings

Timeouts are based on the expected run times of each job, with a reasonable buffer.

### Main Workflow (`rust.yml`)

-   **`test` job (30 minutes):** This job runs the full integration test suite, which is the longest part of the build. The 30-minute timeout provides a sufficient buffer, as the suite typically finishes in about 15 minutes.
-   **`clippy` job (20 minutes):** The linter can be slow on a cold cache; this timeout prevents it from running indefinitely.

### Deployment Workflow (`deploy.yml`)

-   **`deploy` job (25 minutes):** A deployment involves building a container, pushing it to a registry, and waiting for the service to become healthy. This timeout accommodates potential network latency and variance in deployment times.

### Step-Level Timeouts

Timeouts are also applied to specific steps within jobs, particularly those involving network calls or potentially long-running processes.

-   **Health Checks (15 minutes total):** After a deployment, a script repeatedly checks the application's health endpoint. This process is allowed to run for up to 15 minutes (12 attempts with increasing delays) before it is considered a failure.
-   **Database Connectivity (8 minutes total):** The `ensure-db-ready.sh` script attempts to connect to the database 5 times before failing. The entire process is capped at 8 minutes.

## Troubleshooting Timeouts

-   **Job cancelled due to timeout:** If a job is cancelled, review the logs to identify the step that was running. This often points to a network issue or a hung process.
-   **Health checks failed:** This typically indicates that the application failed to start correctly after deployment. Review the application's runtime logs in the hosting environment (e.g., DigitalOcean App Platform) to diagnose the startup error.

These timeout settings should be reviewed periodically to ensure they remain appropriate as the project evolves.

---
*Last Updated: 2025-11-06*