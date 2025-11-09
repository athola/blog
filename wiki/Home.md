# Project Wiki

This wiki provides technical documentation for the Rust-based blog engine, including architecture, development workflows, and operational guides.

## Key Documentation

-   [**Architecture Overview**](wiki/Architecture.md): A high-level look at the system design and how the components interact.
-   [**Development Workflow**](wiki/Development-Workflow.md): A guide to setting up a local environment, running tests, and using the command-line interface.
-   [**Deployment Guide**](DEPLOYMENT.md): Instructions for deploying the application to a production environment on DigitalOcean.
-   [**Security Guide**](SECURITY.md): An overview of the security measures, policies, and best practices implemented in the application.

## Project Overview

This project is a full-stack blog engine built with Rust and the Leptos framework. It is designed to be a high-performance, secure, and self-contained application.

### Core Technologies

-   **Frontend**: Leptos (compiles to WASM) with TailwindCSS for styling.
-   **Backend**: An Axum web server that handles both API requests and server-side rendering (SSR).
-   **Database**: SurrealDB, chosen for its real-time capabilities and embedded-database potential.
-   **Testing**: A three-tier testing strategy (unit, integration, CI-optimized) using `nextest`.
-   **Security**: Automated secret scanning on every commit using Gitleaks, Semgrep, and TruffleHog.
-   **CI/CD**: A security-first GitHub Actions pipeline for automated testing and deployment.

## Quick Links

-   **Repository**: [https://github.com/athola/blog](https://github.com/athola/blog)
-   **Live Demo**: [https://alexthola.com](https://alexthola.com)
