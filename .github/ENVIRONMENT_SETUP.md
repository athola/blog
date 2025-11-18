# GitHub Environment Setup Guide

This guide explains how to configure a GitHub repository for production deployments. The configuration uses GitHub Environments to create a secure, automated workflow that requires manual approval for production pushes.

## 1. Create a `production` Environment

1.  In the GitHub repository, navigate to **Settings** → **Environments**.
2.  Click **New environment** and name it `production`.
3.  Configure a protection rule to require reviewer approval for any deployment to the `main` branch. This prevents accidental deployments from going live.

## 2. Add Repository Secrets

The following secrets must be added in **Settings** → **Secrets and variables** → **Actions** to be used by the GitHub Actions workflows.

### DigitalOcean API Token

-   `DIGITALOCEAN_ACCESS_TOKEN`: A DigitalOcean API token with write permissions. The deployment workflow uses this to interact with the App Platform.

### SurrealDB Credentials

-   `SURREAL_ADDRESS`: The private IP address and port of the database server.
-   `SURREAL_NS`: The target namespace in SurrealDB (e.g., `production`).
-   `SURREAL_DB`: The target database name (e.g., `alexthola_blog`).
-   `SURREAL_USERNAME`: The database username.
-   `SURREAL_PASSWORD`: The database password.

## 3. DigitalOcean Setup

### API Token Generation

The `DIGITALOCEAN_ACCESS_TOKEN` can be generated in the **API** section of the DigitalOcean dashboard. It requires **Write** scope.

### App Platform

The application is designed to be created using the `.do/app.yaml` specification file located in the repository. This file defines the services and configuration for the App Platform, ensuring repeatable setups.

## 4. Pre-Deployment Checklist

Before a deployment can run, ensure that:

-   [ ] All required secrets are configured in GitHub Actions settings.
-   [ ] The DigitalOcean API token is valid and has the correct permissions.
-   [ ] The database server is running and accessible from the App Platform's VPC.

## 5. Troubleshooting

-   **Deployment Skipped:** A skipped deployment usually indicates that it is awaiting approval in the GitHub environment or that a required secret is missing.
-   **Database Connection Issues:** If the application cannot connect to the database, use SSH to access the database server and check the database service status (`systemctl status surrealdb`) and firewall rules (`ufw status`).
