# CI/CD Timeout Configuration

This document outlines the timeout constraints implemented across all GitHub Actions workflows to prevent excessive CI minute consumption.

## Timeout Strategy

### Job-Level Timeouts
All jobs have been configured with appropriate timeout limits based on their expected duration:

#### rust.yml
- **test job**: 30 minutes (includes matrix builds for dev/release)
- **clippy job**: 20 minutes (linting + security audits)

#### migrations.yml
- **migration-syntax-check**: 10 minutes (external PR syntax validation)
- **migration-check**: 15 minutes (trusted PR database validation)

#### deploy.yml
- **deployment-check**: 5 minutes (configuration validation)
- **deploy**: 25 minutes (DigitalOcean deployment + health checks)
- **notify**: 2 minutes (status reporting)

#### claude.yml / claude-code-review.yml
- **claude**: 10 minutes (basic Claude interactions)
- **claude-review**: 15 minutes (comprehensive code reviews)

#### ci-cd.yml
- **pipeline-status**: 2 minutes (status reporting)

### Step-Level Timeouts
Critical operations have individual step timeouts:

#### Build Operations
- **Build workspace**: 20 minutes (Rust compilation can be slow)
- **Security audit**: 5 minutes (cargo audit should be fast)
- **Dependency check**: 10 minutes (cargo udeps can be slower)
- **Benchmarks**: 15 minutes (performance tests vary)
- **WASM size check**: 2 minutes (simple file operations)

#### Network Operations
- **SurrealDB CLI download**: 5 minutes (binary downloads)
- **Migration syntax validation**: 5 minutes (local validation)
- **Database connectivity**: 8 minutes (includes retry logic)
- **Domain accessibility**: 2 minutes (network tests)
- **Health checks**: 15 minutes (post-deployment validation)
- **Core functionality**: 5 minutes (endpoint testing)
- **Performance validation**: 3 minutes (response time checks)

#### Deployment Operations
- **Wait for propagation**: 2 minutes (simple sleep)
- **Post-deployment health**: 15 minutes (comprehensive validation)

### Command-Level Timeouts
Individual commands within scripts have their own timeout limits:

#### Network Commands
- `curl` operations: 10-30 seconds max
- `ping` operations: 30 seconds max
- SSL certificate checks: 15 seconds max
- Health endpoint checks: 10-15 seconds max

#### Build Commands
- `cargo bench`: 10 minutes max (600 seconds)
- Individual compilation: Based on step timeout
- Download operations: 2 minutes max (120 seconds)

#### Database Operations
- SurrealDB connectivity: 10-30 seconds per attempt
- Migration validation: 30 seconds per file

### Retry Logic Limits
Network operations use controlled retry mechanisms:

#### Database Connectivity (migrations.yml)
- **Max attempts**: 5
- **Individual timeout**: 10-30 seconds
- **Max wait between retries**: 60 seconds
- **Total time limit**: 8 minutes (job timeout)
- **Exponential backoff**: Capped to prevent excessive delays

#### Health Checks (deploy.yml)
- **Max attempts**: 12 (reduced from 15)
- **Individual timeout**: 10 seconds
- **Max wait between retries**: 45 seconds
- **Total time limit**: 15 minutes (step timeout)
- **Multiple endpoints**: Fail-fast on first success

#### Download Operations
- **Network retry**: 3 attempts (CARGO_NET_RETRY)
- **Network timeout**: 120 seconds (CARGO_NET_TIMEOUT)
- **Individual downloads**: 120 seconds max

## Fail-Fast Configuration

### Matrix Strategies
```yaml
strategy:
  matrix:
    profile: [dev, release]
  fail-fast: false  # Allow both profiles to complete for better debugging
```

### Critical Path Protection
- Build failures immediately stop dependent jobs
- Security audit failures block deployment
- Database connectivity failures skip migration application
- Health check failures indicate deployment issues

### Non-Critical Operations
- Performance checks are warnings only
- Security header checks are informational
- SSL certificate validation is advisory

## Cost Optimization

### Expected CI Minutes Per Workflow Run

#### Pull Request (Typical)
- **rust.yml**: ~15-25 minutes (both matrix jobs)
- **migrations.yml**: ~5-10 minutes (syntax check only)
- **Total**: ~20-35 minutes per PR

#### Main Branch Push (Full Pipeline)
- **rust.yml**: ~15-25 minutes
- **migrations.yml**: ~10-15 minutes (with database check)
- **deploy.yml**: ~15-25 minutes (with health validation)
- **Total**: ~40-65 minutes per deployment

#### Maximum Possible (All Timeouts Hit)
- **rust.yml**: 50 minutes (30 + 20)
- **migrations.yml**: 25 minutes (10 + 15)
- **deploy.yml**: 32 minutes (5 + 25 + 2)
- **Total**: 107 minutes (worst case scenario)

### Cost Control Measures

1. **Aggressive Timeout Limits**: Prevent runaway jobs
2. **Reduced Retry Counts**: Faster failure detection
3. **Capped Wait Times**: Prevent excessive backoff delays
4. **Selective Execution**: Skip unnecessary operations
5. **Matrix Optimization**: Parallel execution where beneficial

### Monitoring and Alerts

#### Workflow Duration Alerts
- Jobs approaching timeout limits (>80% of timeout)
- Unusual duration patterns (>2x typical time)
- Retry loop detection (multiple consecutive failures)

#### Cost Tracking
- Weekly CI minute usage reports
- Per-workflow cost analysis
- Optimization opportunity identification

## Troubleshooting

### Common Timeout Issues

#### "Job was cancelled due to timeout"
1. Check if the timeout is appropriate for the operation
2. Look for hanging processes or network issues
3. Consider breaking large operations into smaller steps
4. Review retry logic for excessive attempts

#### "Step timed out after X minutes"
1. Identify which specific operation is slow
2. Check network connectivity for downloads/API calls
3. Review build cache effectiveness
4. Consider increasing timeout if operation is legitimately slow

#### "Health checks failed repeatedly"
1. Verify application is actually deployed and running
2. Check domain DNS configuration
3. Review application startup time
4. Validate health endpoint implementation

### Performance Optimization

#### If builds are consistently slow:
1. Review cargo cache configuration
2. Optimize dependency graph
3. Consider selective compilation
4. Use more powerful runner instances

#### If deployments timeout:
1. Check DigitalOcean service status
2. Review application container startup time
3. Optimize health check endpoints
4. Consider staged deployment approach

#### If tests are taking too long:
1. Identify slow test cases
2. Parallelize test execution
3. Use test filtering for incremental runs
4. Consider separate integration test workflow

---

## Maintenance

This timeout configuration should be reviewed quarterly to:
1. Adjust limits based on actual usage patterns
2. Optimize for new tools or dependencies
3. Account for infrastructure improvements
4. Balance speed vs reliability needs

Last updated: 2025-09-06
