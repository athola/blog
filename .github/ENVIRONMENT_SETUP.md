# GitHub Environment Setup

This guide details the steps to configure a GitHub repository for production deployments using GitHub Environments. This process establishes a secure, automated workflow that requires manual approval for production pushes.

## 1. Create a `production` Environment

1.  Navigate to **Settings** → **Environments** in your GitHub repository.
2.  Click **New environment** and name it `production`.
3.  Configure a protection rule to require reviewer approval for deployments to the `main` branch. This prevents accidental deployments to production.

## 2. Configure Repository Secrets

The following secrets must be configured in **Settings** → **Secrets and variables** → **Actions**. These secrets are used by GitHub Actions workflows.

### DigitalOcean API Token

-   `DIGITALOCEAN_ACCESS_TOKEN`: Provide a DigitalOcean API token with **Write** permissions specifically for the App Platform. This token enables the deployment workflow to interact with DigitalOcean.

### SurrealDB Credentials

-   `SURREAL_ADDRESS`: The private IP address and port of the database server.
-   `SURREAL_NS`: The target SurrealDB namespace (e.g., `production`).
-   `SURREAL_DB`: The target database name (e.g., `alexthola_blog`).
-   `SURREAL_USERNAME`: The database username.
-   `SURREAL_PASSWORD`: The database password.

## 3. DigitalOcean Application Setup

### API Token Generation

Generate the `DIGITALOCEAN_ACCESS_TOKEN` in the **API** section of your DigitalOcean dashboard. Ensure the token has **Write** scope for the App Platform.

### App Platform Configuration

Create the application using the `.do/app.yaml` specification file located in this repository. This file defines the services and configuration for the App Platform, ensuring repeatable and consistent deployments.

## 4. Pre-Deployment Checklist

Before initiating a deployment, ensure the following conditions are met:

-   [ ] All required secrets are configured correctly in GitHub Actions settings.
-   [ ] The DigitalOcean API token is valid and has the necessary permissions.
-   [ ] The database server is running and accessible from the App Platform's VPC.

## 5. Troubleshooting Deployment Issues

-   **Deployment Skipped**: This typically indicates either a pending approval within the GitHub environment or a missing required secret.
-   **Database Connection Problems**: If the application cannot connect to the database, SSH into the database server. Check the database service status (`systemctl status surrealdb`) and verify firewall rules (`ufw status`).
