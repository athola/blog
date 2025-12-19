# Production Deployment Guide

This document describes deploying the blog to a DigitalOcean production environment.

## Architecture Overview

-   **Application**: Rust and Leptos application on DigitalOcean App Platform.
-   **Database**: A self-hosted SurrealDB instance on a dedicated DigitalOcean Droplet.
-   **Domain**: `alexthola.com`, managed via NameCheap.
-   **Estimated Monthly Cost**: ~$19 ($5 app, $12 Droplet, $2.40 backups).

## Prerequisites

-   A DigitalOcean account with billing enabled.
-   A registered domain name.
-   The project source code hosted in a GitHub repository.
-   `doctl` (DigitalOcean CLI) installed locally (optional, but recommended).

## Part 1: Database Setup (SurrealDB Droplet)

The recommended setup for the SurrealDB Droplet uses the provided `cloud-init` script, automating installation and initial configuration.

### Droplet Configuration

-   **Image**: Ubuntu 22.04 LTS
-   **Plan**: Basic, $12/month (2GB RAM, 1 vCPU, 50GB SSD). The 1GB plan is not recommended as it can lead to out-of-memory errors.
-   **Region**: New York (NYC3) or the same region as your App Platform application.
-   **Authentication**: Add your SSH key.
-   **User Data**: In the "Advanced Options" section, check "User data" and paste the `cloud-init` script below.

### Cloud-Init Script

This script automates SurrealDB installation, service configuration, and basic security hardening.

```yaml
#cloud-config
package_update: true
package_upgrade: true
packages:
  - curl
  - ufw
  - fail2ban
runcmd:
  - curl -sSf https://install.surrealdb.com | sh
  - useradd -r -s /bin/false surrealdb
  - mkdir -p /var/lib/surrealdb
  - chown surrealdb:surrealdb /var/lib/surrealdb
  - chmod 700 /var/lib/surrealdb
write_files:
  - path: /etc/systemd/system/surrealdb.service
    content: |
      [Unit]
      Description=SurrealDB Database
      After=network.target
      [Service]
      Type=simple
      User=surrealdb
      Group=surrealdb
      ExecStart=/root/.surrealdb/surreal start \
        --bind 0.0.0.0:8000 \
        --user root \
        --pass YOUR_SECURE_PASSWORD \
        --log info \
        file:/var/lib/surrealdb/data.db
      Restart=always
      RestartSec=10
      [Install]
      WantedBy=multi-user.target
system_info:
  default_user:
    name: admin
final_message: "Droplet setup is complete. SSH into the Droplet to set the database password and configure the firewall."
```

### Post-Provisioning Steps

After Droplet creation, SSH in to complete setup.

**1. Set the Database Password**

Generate a secure password and update the service configuration.

```bash
# Generate a password
openssl rand -base64 32

# Edit the service file and replace YOUR_SECURE_PASSWORD
sudo nano /etc/systemd/system/surrealdb.service

# Reload the service to apply changes
sudo systemctl daemon-reload
sudo systemctl restart surrealdb
```

**2. Configure the Firewall**

To secure the database, restrict access to the App Platform's private VPC network.

First, get your app's VPC IP range from the DigitalOcean dashboard under **Your App -> Settings -> VPC Network**.

Then, configure the firewall on the Droplet:

```bash
# Allow SSH access
sudo ufw allow ssh

# Allow database access ONLY from your app's VPC
sudo ufw allow from <YOUR_APP_VPC_RANGE> to any port 8000

# Enable the firewall
sudo ufw enable
```

The database setup is now complete.

### Manual Database Setup (Alternative)

If you prefer manual Droplet configuration, create a Droplet with the listed specifications (without user data) and follow these steps.

**1. Install SurrealDB**

```bash
ssh root@YOUR_DROPLET_IP
curl -sSf https://install.surrealdb.com | sh
```

**2. Create Service User**

```bash
useradd -r -s /bin/false surrealdb
mkdir -p /var/lib/surrealdb
chown surrealdb:surrealdb /var/lib/surrealdb
chmod 700 /var/lib/surrealdb
```

**3. Configure and Start the Service**

Create the systemd service file `/etc/systemd/system/surrealdb.service` with the same content as in the `cloud-init` script, then enable and start the service. Remember to replace `YOUR_SECURE_PASSWORD`.

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now surrealdb
```

## Part 2: Application Deployment

Once the database is running, deploy the application on the DigitalOcean App Platform.

### 1. Create the App

1.  In the DigitalOcean console, navigate to **Apps** and click **Create App**.
2.  Select your GitHub repository (`athola/blog`) and the `master` branch.
3.  Enable **Autodeploy** to automatically redeploy on pushes to `master`.
4.  DigitalOcean will detect the `Dockerfile` and configure the application.
    Ensure the service uses the Dockerfile build type (not a buildpack).

### 2. Configure the App

-   **Name**: A descriptive name, e.g., `blog-web`.
-   **Region**: New York (NYC3) or the same region as your database Droplet.
-   **Instance Size**: Basic, $5/month.
-   **HTTP Port**: Set to `8080`.

### 3. Set Environment Variables

Add the following as **encrypted** environment variables in the app's settings.

```
RUST_LOG=info
LEPTOS_SITE_ADDR=0.0.0.0:8080
LEPTOS_SITE_ROOT=site
LEPTOS_HASH_FILES=true
SURREAL_ADDRESS=http://YOUR_DROPLET_PRIVATE_IP:8000
SURREAL_NS=production
SURREAL_DB=alexthola_blog
SURREAL_USERNAME=root
SURREAL_PASSWORD=YOUR_SECURE_PASSWORD
```

**Note**: Use the Droplet's **private IP** for `SURREAL_ADDRESS` to ensure traffic stays within the VPC. Mark the `SURREAL_PASSWORD` as encrypted.

### 4. Deploy

Click **Create Resources**. The initial build and deployment will take 10-15 minutes.

### 5. Configure DNS

After the app is live, point your domain to it.

1.  In your app's **Settings** tab, go to the **Domains** section and add your custom domain (e.g., `alexthola.com`).
2.  Follow the instructions to configure your DNS records. The recommended approach is to use DigitalOcean's nameservers. In your domain registrar's dashboard (e.g., NameCheap), change the nameservers to `ns1.digitalocean.com`, `ns2.digitalocean.com`, and `ns3.digitalocean.com`.
3.  In the DigitalOcean **Networking** tab, ensure you have an `A` record for your root domain (`@`) pointing to your app, and a `CNAME` record for `www` pointing to the root domain (`@`).

### 6. Initialize the Database

The final step is to apply the database schema.

```bash
# Set environment variables locally
export SURREAL_ADDRESS="http://YOUR_DROPLET_PUBLIC_IP:8000"
export SURREAL_NS="production"
export SURREAL_DB="alexthola_blog"
export SURREAL_USERNAME="root"
export SURREAL_PASSWORD="YOUR_SECURE_PASSWORD"

# Connect to the database and import the schema
surreal sql --conn $SURREAL_ADDRESS --user $SURREAL_USERNAME --pass $SURREAL_PASSWORD --ns $SURREAL_NS --db $SURREAL_DB < migrations/schema.surql
```

**Note**: For this one-time setup, you can temporarily open the firewall to your local IP or run this command from a trusted server. Remember to close the firewall rule afterward.

## Troubleshooting Guide

### SSH Permission Denied

If you cannot SSH into the Droplet (`Permission denied (publickey)`), reset the root password.

1.  In the DigitalOcean console, go to your **Droplet > Access**.
2.  Click **Reset Root Password** and check your email for a temporary password.
3.  Log in as `root` using the temporary password. You will be prompted to set a new one.

To avoid this, always add your SSH key during Droplet creation.

### Cloud-Init Failures

If automated setup fails, check logs to diagnose the issue.

```bash
# Check the status of the cloud-init service
cloud-init status

# Review the output logs for errors
sudo cat /var/log/cloud-init-output.log
```

If necessary, you can run the setup steps manually by following the instructions in the "Manual Database Setup" section.

### Application Fails to Start

Check the application's runtime logs for errors. You can do this in the DigitalOcean console (**App > Logs > Runtime Logs**) or via `doctl`.

```bash
doctl apps logs <YOUR_APP_ID> --type=run --follow
```

Common causes:
-   Missing or incorrect environment variables.
-   Database connection failure (check firewall and `SURREAL_ADDRESS`).
-   Incorrect port binding (the application must bind to `0.0.0.0:8080`).

### Database Connection Errors

1.  **Verify the service is running** on the Droplet.
    ```bash
    ssh root@YOUR_DROPLET_IP
    systemctl status surrealdb
    ```
2.  **Check the service logs** for errors.
    ```bash
    journalctl -u surrealdb -n 50
    ```
3.  **Test the health endpoint**. From the Droplet, you should be able to connect to the database.
    ```bash
    curl http://localhost:8000/health
    ```
    If this fails, the database service is not running correctly. If it succeeds, the issue is likely with the firewall or the private network connection from the App Platform.

### Domain Not Propagating

DNS changes can take time to propagate. Use `dig` to check the status.

```bash
# Should return the IP of your DigitalOcean app
dig your-domain.com +short

# Should return the DigitalOcean nameservers
dig NS your-domain.com +short
```

If the issue persists after several hours, double-check the DNS records in your DigitalOcean networking panel.

## Operations and Maintenance

### Security

-   **Database**: Access restricted by firewall to app's private VPC. Use strong, generated password, rotated quarterly.
-   **Application**: App Platform provides automatic HTTPS, DDoS mitigation, and a managed runtime.
-   **Secrets**: Credentials not stored in repository; managed as encrypted environment variables in App Platform.

### Routine Maintenance

**Monthly Tasks**
-   Review application and database logs for unusual activity.
-   Update dependencies with `cargo update` and run tests before deploying.
-   Verify that database backups are being created successfully.

**Quarterly Tasks**
-   Perform a security audit of the application and its dependencies.
-   Rotate the database password and update the `SURREAL_PASSWORD` environment variable.
-   Review monthly costs and adjust resources as needed.

### Useful Commands

```bash
# View application logs
doctl apps logs <APP_ID> --type=run --follow

# Restart the application
doctl apps restart <APP_ID>

# Create a manual database backup
surreal export --conn $SURREAL_ADDRESS --user $SURREAL_USERNAME --pass $SURREAL_PASSWORD --ns $SURREAL_NS --db $SURREAL_DB backup.surql

# List droplets by tag
doctl compute droplet list --tag-name blog

# Access droplet metadata (from the droplet itself)
curl http://169.254.169.254/metadata/v1/id
```

## Deployment Notes

### Key Learnings

-   **Droplet Sizing**: The $12/month Droplet (2GB RAM) is recommended; the 1GB option can cause out-of-memory errors.
-   **Build Optimization**: The `Dockerfile` has been optimized to reduce build times on the App Platform.
-   **DNS Propagation**: DNS changes can take up to two hours to fully propagate.

### Cost and Scaling

| Service                | Monthly Cost |
| ---------------------- | ------------ |
| App Platform (Basic)   | $5.00        |
| SurrealDB Droplet      | $12.00       |
| Droplet Backups        | $2.40        |
| **Total**              | **$19.40**   |

Consider upgrading resources when:
-   Sustained CPU usage above 70% on the app.
-   Sustained memory usage above 80% on the Droplet.
-   Consistently slow database queries.

---
*Last updated: 2025-11-06*
