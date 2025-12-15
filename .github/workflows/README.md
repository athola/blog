# GitHub Actions Workflows

This directory contains GitHub Actions workflows that automate various CI/CD tasks for the blog project.

## Workflows Overview

### 1. `docker-build-test.yml`

**Purpose**: Tests Docker builds to ensure they work for DigitalOcean deployment.

**Triggers**:
- Pull requests (when Dockerfile or related files change)
- Pushes to master/main/develop branches

**Jobs**:
1. **dockerfile-lint**: Uses Hadolint to check Dockerfile best practices
2. **dockerfile-security**: Scans Dockerfile for security vulnerabilities with Trivy
3. **docker-build**: Builds the Docker image to verify it compiles correctly
4. **docker-build-production**: Builds and tests the production image
5. **deploy-test**: Validates DigitalOcean app.yaml configuration
6. **build-summary**: Provides a summary of all build results

### 2. `ci-docker-pr.yml`

**Purpose**: Fast CI checks for PRs that affect Docker builds.

**Triggers**:
- Pull requests (when Dockerfile or Cargo files change)

**Jobs**:
1. **rust-test**: Runs Rust compilation, formatting, clippy, and tests
2. **verify-docker-cargo**: Ensures Dockerfile copies all workspace members
3. **docker-file-checks**: Validates Dockerfile for required dependencies
4. **ci-summary**: Summarizes CI results

### 3. `rust.yml`

**Purpose**: Comprehensive Rust testing and validation.

**Triggers**:
- Pull requests
- Pushes to main branches

### 4. `deploy.yml`

**Purpose**: Handles deployment to production.

**Triggers**:
- Push to master branch

## How the Workflows Prevent Deployment Issues

### Pre-merge Validation
- **Rust compilation**: Catches code errors before they reach Docker
- **Dockerfile linting**: Ensures Docker follows best practices
- **Security scanning**: Identifies vulnerabilities early
- **Workspace alignment**: Verifies all workspace members are included in Docker

### Docker-specific Checks
- **Required dependencies**: Ensures clang and other build tools are present
- **Ring crate compatibility**: Specifically checks for clang (required for ring)
- **DigitalOcean compatibility**: Validates DO-specific requirements
- **Multi-stage build**: Ensures production image works correctly

### Production Readiness
- **Health checks**: Verifies the application starts and responds
- **Port exposure**: Confirms correct port (8080) is exposed
- **Non-root user**: Ensures security best practices
- **Environment variables**: Tests required env vars are handled

## Best Practices Implemented

1. **Fast feedback**: Quick CI runs on every PR
2. **Comprehensive testing**: Full Docker build on critical paths
3. **Caching**: Uses GitHub Actions cache for faster builds
4. **Security**: Includes vulnerability scanning
5. **Clear reporting**: Summarizes results in PR checks
6. **Parallel execution**: Runs jobs in parallel when possible

## Troubleshooting

### Common Issues

1. **Clang not found**
   - Ensure `clang` is installed in Dockerfile
   - Check that `CC=clang` environment variable is set

2. **Workspace member not found**
   - Verify all workspace members are copied in Dockerfile
   - Check both source and Cargo.toml files are copied

3. **Health check failures**
   - Verify the health check endpoint exists
   - Check that the application starts within the timeout

4. **Security scan failures**
   - Review Trivy scan results
   - Update dependencies if vulnerabilities are found

## Workflow Dependencies

The workflows depend on the following files:
- `Dockerfile`: Main Docker configuration
- `.do/app.yaml`: DigitalOcean configuration
- `Cargo.toml` & `Cargo.lock`: Rust dependencies
- All workspace member directories

## Permissions Required

The workflows require the following GitHub token permissions:
- `contents: read` (default)
- `packages: write` (for container registry)
- `security-events: write` (for security scan results)