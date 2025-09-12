use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{Duration, Instant};

/// Lightweight integration tests for the Leptos development server
///
/// This test suite is optimized for CI environments with limited resources.
/// Tests run sequentially and use reduced timeouts to prevent resource exhaustion.
#[cfg(test)]
mod server_integration_tests_ci {
    use super::*;

    /// Test timeouts (reduced for CI)
    const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Core application pages for testing
    const CORE_PAGES: &[(&str, &str)] = &[
        ("/", "Home page"),
        ("/references", "References page"),
    ];

    /// Critical assets that must be available
    const CRITICAL_ASSETS: &[(&str, &str, u64)] = &[
        ("/pkg/blog.css", "text/css", 512),
        ("/pkg/blog.js", "text/javascript", 512),
    ];

    /// Port counter for isolated test instances
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(3020);

    /// Test server instance that runs for the duration of a single test
    struct TestServer {
        process: Option<Child>,
        client: reqwest::Client,
        db_process: Option<Child>, // Track the database process
        port: u16,
    }

    impl TestServer {
        /// Start a test development server
        async fn start() -> Result<Self, Box<dyn std::error::Error>> {
            // Get a unique port for this test instance
            let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let server_url = format!("http://127.0.0.1:{}", port);
            
            eprintln!("Starting CI test server on port {}...", port);
            Self::cleanup_existing_processes(port).await;

            Self::ensure_ports_available(port).await?;

            // Start database and wait for it to be ready
            let db_process = Self::start_database(port).await?;

            // Give database extra time to fully initialize
            tokio::time::sleep(Duration::from_millis(500)).await;

            eprintln!("Starting Leptos development server on port {}...", port);

            // Build the server first to ensure it's up to date
            eprintln!("Building server in debug mode...");
            let build_status = Command::new("cargo")
                .args(["build", "-p", "server"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|e| format!("Failed to build server: {}", e))?;

            if !build_status.success() {
                return Err("Failed to build server".into());
            }

            // Calculate the database port
            let db_port = 8000 + (port - 3007); // Use a unique DB port for each test

            // Set environment variables for the server
            std::env::set_var("LEPTOS_SITE_ADDR", format!("127.0.0.1:{}", port));
            std::env::set_var("SURREAL_HOST", format!("127.0.0.1:{}", db_port));
            
            eprintln!("Starting server binary on port {} with DB on port {}...", port, db_port);
            let mut process = Command::new("./target/debug/server")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .env("LEPTOS_SITE_ADDR", format!("127.0.0.1:{}", port))
                .env("SURREAL_HOST", format!("127.0.0.1:{}", db_port))
                .spawn()
                .map_err(|e| format!("Failed to start server binary: {}", e))?;

            let client = Self::create_client()?;

            Self::wait_for_server_startup(&client, &server_url, &mut process).await?;

            Ok(TestServer {
                process: Some(process),
                client,
                db_process: Some(db_process),
                port,
            })
        }

        /// Start the database and wait for it to be ready
        async fn start_database(port: u16) -> Result<Child, Box<dyn std::error::Error>> {
            let db_port = 8000 + (port - 3007); // Use a unique DB port for each test
            let db_file = format!("rustblog_ci_test_{}.db", port);
            
            eprintln!("Starting SurrealDB database on port {} with file {}...", db_port, db_file);

            // Kill any existing database processes first
            let _ = Command::new("pkill").args(["-f", &format!("surreal.*{}", db_port)]).output();
            // Also kill any processes using the specific db port
            let _ = Command::new("bash").args(["-c", &format!("lsof -ti:{} | xargs -r kill -TERM 2>/dev/null || true", db_port)]).output();
            tokio::time::sleep(Duration::from_millis(250)).await;

            // Start the database process with unique port and file
            let db_command = format!("env SURREAL_EXPERIMENTAL_GRAPHQL=true surreal start --log warn --user root --pass root --bind 127.0.0.1:{} surrealkv:{}", db_port, db_file);
            eprintln!("Executing command: {}", db_command);
            
            let mut db_process = Command::new("bash")
                .arg("-c")
                .arg(&db_command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to start database: {}", e))?;

            // Give the process a moment to start
            tokio::time::sleep(Duration::from_millis(250)).await;

            // Check if the process is still running
            if let Ok(Some(status)) = db_process.try_wait() {
                eprintln!("Database process exited immediately with status: {}", status);
                // Try to get stderr output
                if let Some(ref mut stderr) = db_process.stderr {
                    use std::io::Read;
                    let mut buffer = String::new();
                    let _ = stderr.read_to_string(&mut buffer);
                    if !buffer.is_empty() {
                        eprintln!("Database stderr: {}", buffer);
                    }
                }
                return Err("Database process failed to start".into());
            }

            // Wait for database to be ready
            let timeout = Instant::now() + Duration::from_secs(20);

            eprintln!("Waiting for database on port {} to be ready (up to 20 seconds)...", db_port);

            while Instant::now() < timeout {
                if Self::test_database_connection(db_port).await {
                    eprintln!("Database on port {} is ready!", db_port);
                    // Give it a bit more time to fully initialize
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    return Ok(db_process);
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }

            // If we timed out, try to get database logs for debugging
            if let Some(ref mut stderr) = db_process.stderr {
                use std::io::Read;
                let mut buffer = String::new();
                let _ = stderr.read_to_string(&mut buffer);
                if !buffer.is_empty() {
                    eprintln!("Database stderr after timeout: {}", buffer);
                }
            }

            Err(format!("Database on port {} is not responsive within timeout", db_port).into())
        }

        /// Test if database is responsive
        async fn test_database_connection(port: u16) -> bool {
            // Try to make an actual HTTP request to SurrealDB's root endpoint
            let client = reqwest::Client::new();
            let db_url = format!("http://127.0.0.1:{}", port);
            
            // First try a simple TCP connection to see if the port is open
            match tokio::time::timeout(Duration::from_millis(500), 
                tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))).await {
                Ok(Ok(_)) => {
                    eprintln!("Database port {} is open", port);
                }
                Ok(Err(e)) => {
                    eprintln!("Database port {} connection failed: {}", port, e);
                    return false;
                }
                Err(e) => {
                    eprintln!("Database port {} connection timed out: {}", port, e);
                    return false;
                }
            }
            
            // Then try an HTTP request with more detailed error handling
            match client
                .get(&db_url)
                .timeout(Duration::from_secs(1))
                .send()
                .await
            {
                Ok(response) => {
                    // Database is ready if we get any response
                    eprintln!("Database HTTP test response on port {}: {} (status: {})", 
                        port, response.status(), response.status());
                    true
                }
                Err(e) => {
                    eprintln!("Database HTTP test failed on port {}: {}", port, e);
                    false
                }
            }
        }

        /// Create HTTP client with standard configuration
        fn create_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
            Ok(reqwest::Client::builder().timeout(CLIENT_TIMEOUT).build()?)
        }

        /// Ensure required ports are available or clean them up
        async fn ensure_ports_available(port: u16) -> Result<(), Box<dyn std::error::Error>> {
            // Check server ports (our unique port, and the reload port)
            let reload_port = port + 1000; // Use a unique reload port
            let db_port = 8000 + (port - 3007); // Use a unique DB port
            let server_ports = [port, reload_port, db_port];
            let ports_in_use = server_ports
                .iter()
                .filter(|&&p| Self::is_port_in_use(p))
                .collect::<Vec<_>>();

            if !ports_in_use.is_empty() {
                // Try to clean up server processes
                let _ = Command::new("pkill").args(["-f", &format!("server.*{}", port)]).output();
                // Kill any processes using the server ports
                let ports_str = server_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
                let _ = Command::new("bash").args(["-c", &format!("lsof -ti:{} | xargs -r kill -TERM 2>/dev/null || true", ports_str)]).output();
                tokio::time::sleep(Duration::from_millis(250)).await;

                // Check again after cleanup
                let still_in_use = server_ports
                    .iter()
                    .filter(|&&p| Self::is_port_in_use(p))
                    .collect::<Vec<_>>();

                if !still_in_use.is_empty() {
                    return Err(format!(
                        "Server ports still in use after cleanup: {:?}",
                        still_in_use
                    )
                    .into());
                }
            }
            Ok(())
        }

        /// Wait for server to start and respond
    async fn wait_for_server_startup(
        client: &reqwest::Client,
        server_url: &str,
        process: &mut Child,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timeout = Instant::now() + Duration::from_secs(30); // Reduced timeout for CI
        let mut attempt = 0;

        eprintln!("Waiting for Leptos server on {} to respond...", server_url);

        while Instant::now() < timeout {
            attempt += 1;

            // Check if the process has exited unexpectedly
            match process.try_wait() {
                Ok(Some(status)) => {
                    eprintln!("Server process exited unexpectedly with status: {}", status);
                    // Try to get stderr output if available
                    if let Some(ref mut stderr) = process.stderr {
                        use std::io::Read;
                        let mut buffer = String::new();
                        let _ = stderr.read_to_string(&mut buffer);
                        if !buffer.is_empty() {
                            eprintln!("Server stderr: {}", buffer);
                        }
                    }
                    return Err(format!("Server process exited unexpectedly: {}", status).into());
                }
                Ok(None) => {
                    // Process is still running
                }
                Err(e) => {
                    eprintln!("Error checking server process status: {}", e);
                }
            }

            match client.get(server_url).send().await {
                Ok(response) if response.status().is_success() => {
                    eprintln!("Server on {} is responding! (attempt {})", server_url, attempt);
                    // Give it a moment to fully initialize
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    return Ok(());
                }
                Ok(response) => {
                    eprintln!(
                        "Server on {} responded with status: {} (attempt {})",
                        server_url,
                        response.status(),
                        attempt
                    );
                }
                Err(e) => {
                    if attempt % 3 == 0 { // Log more frequently but still limit output
                        eprintln!("Connection attempt {} to {}: {}", attempt, server_url, e);
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        // Before giving up, check if the process is still running
        match process.try_wait() {
            Ok(Some(status)) => {
                eprintln!("Server process exited with status: {} after timeout", status);
                return Err(format!("Server process exited with status: {} after timeout", status).into());
            }
            Ok(None) => {
                eprintln!("Server process still running but timed out waiting for response");
            }
            Err(e) => {
                eprintln!("Error checking server process after timeout: {}", e);
            }
        }

        Err(format!("Server on {} failed to start within timeout period", server_url).into())
    }

        /// Clean up existing processes
        async fn cleanup_existing_processes(port: u16) {
            let db_port = 8000 + (port - 3007);
            let reload_port = port + 1000;
            
            // Kill processes associated with our specific ports
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!("lsof -ti:{},{},{} | xargs -r kill -TERM 2>/dev/null || true", port, reload_port, db_port),
                ])
                .output();

            // Wait a bit for termination
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Force kill if still running
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!("lsof -ti:{},{},{} | xargs -r kill -KILL 2>/dev/null || true", port, reload_port, db_port),
                ])
                .output();
        }

        /// Check if a port is in use
        fn is_port_in_use(port: u16) -> bool {
            TcpListener::bind(("127.0.0.1", port)).is_err()
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            eprintln!("Cleaning up CI TestServer on port {}...", self.port);
            
            // Clean up the server process
            if let Some(mut process) = self.process.take() {
                eprintln!("Terminating server process on port {}...", self.port);
                
                // Try graceful termination first
                match process.kill() {
                    Ok(_) => eprintln!("Sent kill signal to server process on port {}", self.port),
                    Err(e) => eprintln!("Failed to send kill signal to server process on port {}: {}", self.port, e),
                }

                // Wait for process to terminate with timeout
                let start = std::time::Instant::now();
                let timeout = std::time::Duration::from_millis(500);

                while start.elapsed() < timeout {
                    match process.try_wait() {
                        Ok(Some(status)) => {
                            eprintln!("Server process on port {} exited with status: {}", self.port, status);
                            break;
                        }
                        Ok(None) => {
                            // Still running, continue waiting
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                        Err(e) => {
                            eprintln!("Error checking server process status on port {}: {}", self.port, e);
                            break;
                        }
                    }
                }

                // Force kill if still running
                if let Ok(None) = process.try_wait() {
                    eprintln!("Force killing server process on port {}...", self.port);
                    let _ = process.kill();
                    let _ = process.wait();
                }
            }

            // Clean up the database process
            if let Some(mut db_process) = self.db_process.take() {
                eprintln!("Terminating database process for port {}...", self.port);
                
                // Try graceful termination first
                match db_process.kill() {
                    Ok(_) => eprintln!("Sent kill signal to database process for port {}", self.port),
                    Err(e) => eprintln!("Failed to send kill signal to database process for port {}: {}", self.port, e),
                }

                // Wait for process to terminate with timeout
                let start = std::time::Instant::now();
                let timeout = std::time::Duration::from_millis(500);

                while start.elapsed() < timeout {
                    match db_process.try_wait() {
                        Ok(Some(status)) => {
                            eprintln!("Database process for port {} exited with status: {}", self.port, status);
                            break;
                        }
                        Ok(None) => {
                            // Still running, continue waiting
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                        Err(e) => {
                            eprintln!("Error checking database process status for port {}: {}", self.port, e);
                            break;
                        }
                    }
                }

                // Force kill if still running
                if let Ok(None) = db_process.try_wait() {
                    eprintln!("Force killing database process for port {}...", self.port);
                    let _ = db_process.kill();
                    let _ = db_process.wait();
                }
                
                // Clean up database file
                let db_file = format!("rustblog_ci_test_{}.db", self.port);
                let _ = std::fs::remove_file(&db_file);
            }

            eprintln!("CI TestServer cleanup completed for port {}.", self.port);
        }
    }

    // === Helper Functions ===

    /// Start a test server for a test
    async fn start_test_server() -> Result<(TestServer, String), Box<dyn std::error::Error>> {
        let server = TestServer::start().await?;
        let server_url = format!("http://127.0.0.1:{}", server.port);
        Ok((server, server_url))
    }

    /// Helper to fetch and validate a page
    async fn fetch_and_validate_page(
        client: &reqwest::Client,
        server_url: &str,
        path: &str,
        description: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let response = client
            .get(format!("{}{}", server_url, path))
            .send()
            .await?;

        assert!(
            response.status().is_success(),
            "{} should return success, got: {}",
            description,
            response.status()
        );
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );

        let body = response.text().await?;
        assert!(
            body.contains("<!DOCTYPE html"),
            "{} should contain HTML doctype",
            description
        );

        Ok(body)
    }

    /// Helper to validate asset serving
    async fn validate_asset(
        client: &reqwest::Client,
        server_url: &str,
        path: &str,
        expected_content_type: &str,
        min_size: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = client
            .get(format!("{}{}", server_url, path))
            .send()
            .await?;

        assert!(
            response.status().is_success(),
            "Asset {} should return success, got: {}",
            path,
            response.status()
        );

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.starts_with(expected_content_type),
            "Asset {} should have content-type {}, got: {}",
            path,
            expected_content_type,
            content_type
        );

        let content_length = response.content_length().unwrap_or(0);
        assert!(
            content_length >= min_size,
            "Asset {} should be at least {} bytes, got: {}",
            path,
            min_size,
            content_length
        );

        Ok(())
    }

    // === Test Cases ===

    /// Test 1: Server Connectivity and Basic Response
    /// Verifies server starts, responds to requests, and returns proper content type
    #[tokio::test]
    #[cfg(feature = "ci")]
    async fn test_server_connectivity() -> Result<(), Box<dyn std::error::Error>> {
        let (server, server_url) = start_test_server().await?;
        let client = server.client.clone();
        let response = client.get(&server_url).send().await?;

        assert!(
            response.status().is_success(),
            "Server should respond with success status, got: {}",
            response.status()
        );
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );

        Ok(())
    }

    /// Test 2: Page Navigation and Content
    /// Tests core pages for accessibility, content type, and expected content
    #[tokio::test]
    #[cfg(feature = "ci")]
    async fn test_page_navigation_and_content() -> Result<(), Box<dyn std::error::Error>> {
        let (server, server_url) = start_test_server().await?;
        let client = server.client.clone();

        for &(path, description) in CORE_PAGES {
            let body = fetch_and_validate_page(&client, &server_url, path, description).await?;

            // All pages should contain navigation elements
            assert!(
                body.contains("blog"),
                "{} should contain navigation elements",
                description
            );
        }

        Ok(())
    }

    /// Test 3: Static Asset Serving
    /// Validates that critical assets are served correctly
    #[tokio::test]
    #[cfg(feature = "ci")]
    async fn test_static_asset_serving() -> Result<(), Box<dyn std::error::Error>> {
        let (server, server_url) = start_test_server().await?;
        let client = server.client.clone();

        // Test critical assets
        for &(path, expected_content_type, min_size) in CRITICAL_ASSETS {
            validate_asset(&client, &server_url, path, expected_content_type, min_size).await?;
        }

        Ok(())
    }

    /// Test 4: Error Handling
    /// Tests server behavior with invalid routes
    #[tokio::test]
    #[cfg(feature = "ci")]
    async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let (server, server_url) = start_test_server().await?;
        let client = server.client.clone();

        // Test non-existent route - should still return HTML (SPA routing)
        let response = client
            .get(format!("{}/nonexistent", server_url))
            .send()
            .await?;
        let body = response.text().await?;

        assert!(
            body.contains("<!DOCTYPE html"),
            "Even non-existent routes should return HTML structure"
        );

        Ok(())
    }
}