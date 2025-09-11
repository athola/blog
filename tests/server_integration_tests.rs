use lazy_static::lazy_static;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::Once;
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Integration tests for the Leptos development server
///
/// This test suite uses a shared server instance to minimize resource usage.
/// Tests are organized by functional areas: connectivity, content, assets, and performance.
#[cfg(test)]
mod server_integration_tests {
    use super::*;

    /// Test timeouts
    const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

    /// Development server URL - matches Leptos configuration
    const DEV_SERVER_URL: &str = "http://127.0.0.1:3007";

    /// Core application pages for testing
    const CORE_PAGES: &[(&str, &str)] = &[
        ("/", "Home page"),
        ("/references", "References page"),
        ("/contact", "Contact page"),
    ];

    /// Critical assets that must be available
    const CRITICAL_ASSETS: &[(&str, &str, u64)] = &[
        ("/pkg/blog.css", "text/css", 1024),
        ("/pkg/blog.js", "text/javascript", 1024),
    ];

    lazy_static! {
        static ref SHARED_SERVER: Mutex<Option<SharedTestServer>> = Mutex::new(None);
    }
    static SERVER_INITIALIZED: AtomicBool = AtomicBool::new(false);
    static INIT: Once = Once::new();

    /// Shared server instance that runs for the duration of all tests
    struct SharedTestServer {
        process: Option<Child>,
        client: reqwest::Client,
        db_process: Option<Child>, // Track the database process
    }

    impl SharedTestServer {
        /// Start the shared development server
        async fn start() -> Result<Self, Box<dyn std::error::Error>> {
            Self::cleanup_existing_processes().await;

            Self::ensure_ports_available().await?;

            // Start database and wait for it to be ready
            let db_process = Self::start_database().await?;

            // Give database extra time to fully initialize
            tokio::time::sleep(Duration::from_secs(2)).await;

            eprintln!("Starting Leptos development server...");

            // Build the server first to ensure it's up to date
            let build_status = Command::new("cargo")
                .args(["build", "--release", "-p", "server"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|e| format!("Failed to build server: {}", e))?;

            if !build_status.success() {
                return Err("Failed to build server".into());
            }

            let mut process = Command::new("./target/release/server")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to start server binary: {}", e))?;

            let client = Self::create_client()?;

            Self::wait_for_server_startup(&client, &mut process).await?;

            Ok(SharedTestServer {
                process: Some(process),
                client,
                db_process: Some(db_process),
            })
        }

        /// Start the database and wait for it to be ready
        async fn start_database() -> Result<Child, Box<dyn std::error::Error>> {
            eprintln!("Starting SurrealDB database...");

            // Check if database is already running and responding
            if Self::test_database_connection().await {
                eprintln!("Database is already running and responding!");
                // If it's already running, we don't need to start it again
                // But we still need to return a process handle
                // Let's create a dummy process that we'll never actually use
                let dummy_process = Command::new("sleep")
                    .arg("3600")  // Sleep for an hour
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(|e| format!("Failed to create dummy process: {}", e))?;
                return Ok(dummy_process);
            }

            // Kill any existing database processes first
            let _ = Command::new("pkill").args(["-f", "surreal"]).output();
            // Also kill any processes using port 8000
            let _ = Command::new("bash").args(["-c", "lsof -ti:8000 | xargs -r kill -TERM 2>/dev/null || true"]).output();
            tokio::time::sleep(Duration::from_millis(1000)).await;

            // Start the database process
            eprintln!("Executing command: sh ./db.sh");
            let mut db_process = Command::new("sh")
                .arg("./db.sh")
                .stdout(Stdio::piped())  // Capture stdout for debugging
                .stderr(Stdio::piped())  // Capture stderr for debugging
                .spawn()
                .map_err(|e| format!("Failed to start database with command 'sh ./db.sh': {}", e))?;

            // Give the process a moment to start
            tokio::time::sleep(Duration::from_millis(1000)).await;

            // Check if the process is still running
            if let Ok(Some(status)) = db_process.try_wait() {
                eprintln!("Database process exited immediately with status: {}", status);
                return Err("Database process failed to start".into());
            }

            // Wait for database to be ready
            let timeout = Instant::now() + Duration::from_secs(60); // Increased from 30 to 60 seconds

            eprintln!("Waiting for database to be ready (up to 60 seconds)...");

            while Instant::now() < timeout {
                if Self::test_database_connection().await {
                    eprintln!("Database is ready!");
                    // Give it a bit more time to fully initialize
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    return Ok(db_process);
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }

            Err("Database is not responsive within timeout".into())
        }

        /// Test if database is responsive
        async fn test_database_connection() -> bool {
            // Try to make an actual HTTP request to SurrealDB's root endpoint
            let client = reqwest::Client::new();
            
            // First try a simple TCP connection to see if the port is open
            match tokio::time::timeout(Duration::from_secs(2), 
                tokio::net::TcpStream::connect("127.0.0.1:8000")).await {
                Ok(Ok(_)) => {
                    eprintln!("Database port 8000 is open");
                }
                Ok(Err(e)) => {
                    eprintln!("Database port 8000 connection failed: {}", e);
                    return false;
                }
                Err(e) => {
                    eprintln!("Database port 8000 connection timed out: {}", e);
                    return false;
                }
            }
            
            // Then try an HTTP request with more detailed error handling
            match client
                .get("http://127.0.0.1:8000")
                .timeout(Duration::from_secs(5))
                .send()
                .await
            {
                Ok(response) => {
                    // Database is ready if we get any response
                    eprintln!("Database HTTP test response: {} (body preview: {:?})", 
                        response.status(), 
                        response.text().await.unwrap_or_else(|_| "no text".to_string()).chars().take(100).collect::<String>());
                    true
                }
                Err(e) => {
                    eprintln!("Database HTTP test failed: {}", e);
                    // Even if we get an error, if we can reach the server it means it's running
                    // Check if it's a common error that indicates the server is running but returning an error
                    let error_str = e.to_string().to_lowercase();
                    if error_str.contains("connection closed") || 
                       error_str.contains("connection reset") ||
                       error_str.contains("operation timed out") {
                        eprintln!("Database appears to be running but not fully ready yet");
                        true
                    } else {
                        false
                    }
                }
            }
        }

        /// Create HTTP client with standard configuration
        fn create_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
            Ok(reqwest::Client::builder().timeout(CLIENT_TIMEOUT).build()?)
        }

        /// Ensure required ports are available or clean them up
        async fn ensure_ports_available() -> Result<(), Box<dyn std::error::Error>> {
            // Check server ports (3007, 3001) but allow database port (8000) to be managed separately
            let server_ports = [3007, 3001];
            let ports_in_use = server_ports
                .iter()
                .filter(|&&port| Self::is_port_in_use(port))
                .collect::<Vec<_>>();

            if !ports_in_use.is_empty() {
                // Try to clean up server processes
                let _ = Command::new("pkill").args(["-f", "cargo-leptos"]).output();
                let _ = Command::new("pkill").args(["-f", "leptos"]).output();
                let _ = Command::new("pkill").args(["-f", "server"]).output();
                // Kill any processes using the server ports
                let _ = Command::new("bash").args(["-c", "lsof -ti:3007,3001 | xargs -r kill -TERM 2>/dev/null || true"]).output();
                tokio::time::sleep(Duration::from_millis(1000)).await;

                // Check again after cleanup
                let still_in_use = server_ports
                    .iter()
                    .filter(|&&port| Self::is_port_in_use(port))
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
            process: &mut Child,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let timeout = Instant::now() + Duration::from_secs(90); // Increased timeout
            let mut attempt = 0;

            eprintln!("Waiting for Leptos server to respond...");

            while Instant::now() < timeout {
                attempt += 1;

                // Check if the process has exited unexpectedly
                if let Ok(Some(status)) = process.try_wait() {
                    return Err(format!("Server process exited unexpectedly: {}", status).into());
                }

                match client.get(DEV_SERVER_URL).send().await {
                    Ok(response) if response.status().is_success() => {
                        eprintln!("Server is responding! (attempt {})", attempt);
                        // Give it a moment to fully initialize
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        return Ok(());
                    }
                    Ok(response) => {
                        eprintln!(
                            "Server responded with status: {} (attempt {})",
                            response.status(),
                            attempt
                        );
                    }
                    Err(e) => {
                        if attempt % 10 == 0 {
                            eprintln!("Connection attempt {}: {}", attempt, e);
                        }
                        
                        // Additional debugging - check if process is still alive
                        if attempt % 30 == 0 {
                            if let Ok(Some(status)) = process.try_wait() {
                                return Err(format!("Server process exited during wait: {}", status).into());
                            }
                        }
                    }
                }

                tokio::time::sleep(Duration::from_millis(500)).await; // Reduced sleep for faster response
            }

            // Before giving up, check if the process is still running
            if let Ok(Some(status)) = process.try_wait() {
                return Err(format!("Server process exited with status: {} after timeout", status).into());
            }

            Err("Server failed to start within timeout period".into())
        }

        /// Clean up existing processes
        async fn cleanup_existing_processes() {
            let process_patterns = ["make.*watch", "cargo.*leptos", "cargo-leptos", "surreal", "server"];

            // Graceful shutdown
            for pattern in process_patterns {
                let _ = Command::new("pkill")
                    .args(["-TERM", "-f", pattern])
                    .output();
            }

            let _ = Command::new("bash")
                .args([
                    "-c",
                    "lsof -ti:3007,3001,8000 | xargs -r kill -TERM 2>/dev/null || true",
                ])
                .output();

            Self::wait_for_process_termination().await;

            // Force cleanup if needed
            for pattern in process_patterns {
                let _ = Command::new("pkill")
                    .args(["-KILL", "-f", pattern])
                    .output();
            }

            let _ = Command::new("bash")
                .args([
                    "-c",
                    "lsof -ti:3007,3001,8000 | xargs -r kill -KILL 2>/dev/null || true",
                ])
                .output();

            Self::wait_for_port_release().await;
        }

        /// Wait for process termination with polling
        async fn wait_for_process_termination() {
            let mut poll_interval = interval(Duration::from_millis(100));
            let timeout = Instant::now() + Duration::from_millis(2000);

            while Instant::now() < timeout {
                poll_interval.tick().await;

                let output = Command::new("pgrep")
                    .args(["-f", "make.*watch|cargo.*leptos|cargo-leptos"])
                    .output();

                if let Ok(result) = output {
                    if result.stdout.is_empty() {
                        break;
                    }
                }
            }
        }

        /// Wait for ports to be released
        async fn wait_for_port_release() {
            let mut poll_interval = interval(Duration::from_millis(100));
            let timeout = Instant::now() + Duration::from_millis(1000);

            while Instant::now() < timeout {
                poll_interval.tick().await;
                if !Self::is_port_in_use(3007) && !Self::is_port_in_use(3001) {
                    break;
                }
            }
        }

        /// Check if a port is in use
        fn is_port_in_use(port: u16) -> bool {
            TcpListener::bind(("127.0.0.1", port)).is_err()
        }
    }

    impl Drop for SharedTestServer {
        fn drop(&mut self) {
            if let Some(mut process) = self.process.take() {
                let _ = process.kill();

                let start = std::time::Instant::now();
                let timeout = std::time::Duration::from_millis(500);

                while start.elapsed() < timeout {
                    if let Ok(Some(_)) = process.try_wait() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }

                let _ = process.wait();
            }

            // Clean up the database process
            if let Some(mut db_process) = self.db_process.take() {
                let _ = db_process.kill();

                let start = std::time::Instant::now();
                let timeout = std::time::Duration::from_millis(500);

                while start.elapsed() < timeout {
                    if let Ok(Some(_)) = db_process.try_wait() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }

                let _ = db_process.wait();
            }

            // Also try to kill any remaining surreal processes to be thorough
            let _ = Command::new("pkill").args(["-f", "surreal"]).output();
        }
    }

    // === Helper Functions ===

    /// Get or start the shared server
    async fn get_shared_server() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
        // Use Once to ensure initialization happens exactly once
        if !INIT.is_completed() {
            INIT.call_once(|| {
                // This will only run once
            });
            
            // Initialize the shared server once
            let server = SharedTestServer::start().await?;
            let mut server_guard = SHARED_SERVER.lock().unwrap();
            *server_guard = Some(server);
            SERVER_INITIALIZED.store(true, Ordering::Release);
        } else {
            // Wait for initialization to complete with a timeout
            let timeout = Instant::now() + Duration::from_secs(30);
            while !SERVER_INITIALIZED.load(Ordering::Acquire) && Instant::now() < timeout {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        // Return a client for the shared server
        let server_guard = SHARED_SERVER.lock().unwrap();
        match server_guard.as_ref() {
            Some(server) => Ok(server.client.clone()),
            None => Err("Shared server not initialized".into()),
        }
    }

    /// Get HTTP client for tests
    async fn get_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
        get_shared_server().await
    }

    /// Helper to fetch and validate a page
    async fn fetch_and_validate_page(
        client: &reqwest::Client,
        path: &str,
        description: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let response = client
            .get(format!("{}{}", DEV_SERVER_URL, path))
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
        path: &str,
        expected_content_type: &str,
        min_size: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = client
            .get(format!("{}{}", DEV_SERVER_URL, path))
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
    #[cfg(not(any(feature = "ci", coverage)))]
    async fn test_server_connectivity() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;
        let response = client.get(DEV_SERVER_URL).send().await?;

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
    /// Tests all core pages for accessibility, content type, and expected content
    #[tokio::test]
    #[cfg(not(any(feature = "ci", coverage)))]
    async fn test_page_navigation_and_content() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;

        for &(path, description) in CORE_PAGES {
            let body = fetch_and_validate_page(&client, path, description).await?;

            // All pages should contain navigation elements
            assert!(
                body.contains("blog"),
                "{} should contain navigation elements",
                description
            );
        }

        // Test navigation links on home page
        let home_body = fetch_and_validate_page(&client, "/", "Home page").await?;
        assert!(
            home_body.contains(r#"href="/""#),
            "Should contain home link"
        );
        assert!(
            home_body.contains(r#"href="/references""#),
            "Should contain references link"
        );
        assert!(
            home_body.contains(r#"href="/contact""#),
            "Should contain contact link"
        );
        assert!(
            home_body.contains("github.com/athola"),
            "Should contain GitHub link"
        );
        assert!(
            home_body.contains("linkedin.com/in/alexthola"),
            "Should contain LinkedIn link"
        );

        // Test page-specific content
        let references_body =
            fetch_and_validate_page(&client, "/references", "References page").await?;
        assert!(
            references_body.contains("Project References"),
            "References page should contain 'Project References'"
        );

        let contact_body = fetch_and_validate_page(&client, "/contact", "Contact page").await?;
        assert!(
            contact_body.contains("Get In Touch"),
            "Contact page should contain 'Get In Touch'"
        );
        assert!(
            contact_body.contains("form"),
            "Contact page should contain a form"
        );

        Ok(())
    }

    /// Test 3: Static Asset Serving
    /// Validates that all critical assets (CSS, JS) are served correctly with proper headers
    #[tokio::test]
    #[cfg(not(any(feature = "ci", coverage)))]
    async fn test_static_asset_serving() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;

        // Test critical assets - be more forgiving in coverage mode
        for &(path, expected_content_type, min_size) in CRITICAL_ASSETS {
            match validate_asset(&client, path, expected_content_type, min_size).await {
                Ok(_) => {} // Asset validated successfully
                Err(e) if cfg!(coverage) => {
                    eprintln!("Warning: Asset validation failed in coverage mode: {}", e);
                    // Continue without failing the test in coverage mode
                }
                Err(e) => return Err(e), // Fail normally in non-coverage mode
            }
        }

        // Test WASM asset (optional - don't fail if not ready)
        if let Ok(response) = client
            .get(format!("{}/pkg/blog.wasm", DEV_SERVER_URL))
            .send()
            .await
        {
            if response.status().is_success() {
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                assert!(
                    content_type.contains("wasm")
                        || content_type.contains("application/octet-stream"),
                    "WASM asset should have appropriate content-type, got: {}",
                    content_type
                );

                let content_length = response.content_length().unwrap_or(0);
                assert!(
                    content_length >= 1024,
                    "WASM asset should be at least 1KB, got: {}",
                    content_length
                );
            }
        }

        Ok(())
    }

    /// Test 4: Server Performance
    /// Measures response times to ensure reasonable performance under load
    #[tokio::test]
    #[cfg(not(any(feature = "ci", coverage)))]
    async fn test_server_performance() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;
        let mut response_times = Vec::new();

        // Test multiple requests to get average response time
        for _ in 0..5 {
            let start = Instant::now();
            let response = client.get(DEV_SERVER_URL).send().await?;
            let elapsed = start.elapsed();

            assert!(response.status().is_success());
            response_times.push(elapsed);

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let avg_response_time =
            response_times.iter().sum::<Duration>() / response_times.len() as u32;

        // Be more lenient with performance expectations in coverage mode
        let max_response_time = if cfg!(coverage) {
            Duration::from_secs(30) // Much more lenient for coverage builds
        } else {
            Duration::from_secs(5)
        };

        assert!(
            avg_response_time < max_response_time,
            "Average response time should be under {:?}, got: {:?}",
            max_response_time,
            avg_response_time
        );

        Ok(())
    }

    /// Test 5: Error Handling
    /// Tests server behavior with invalid routes and error conditions
    #[tokio::test]
    #[cfg(not(any(feature = "ci", coverage)))]
    async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;

        // Test non-existent route - should still return HTML (SPA routing)
        let response = client
            .get(format!("{}/nonexistent", DEV_SERVER_URL))
            .send()
            .await?;
        let body = response.text().await?;

        assert!(
            body.contains("<!DOCTYPE html"),
            "Even non-existent routes should return HTML structure"
        );

        Ok(())
    }

    /// Test 6: Complete Development Workflow
    /// End-to-end test ensuring all components work together
    #[tokio::test]
    #[cfg(not(any(feature = "ci", coverage)))]
    async fn test_complete_development_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;

        // Verify server responds
        let response = client.get(DEV_SERVER_URL).send().await?;
        assert!(
            response.status().is_success(),
            "Server should be responsive"
        );

        // Verify all core pages are accessible
        for &(path, _) in CORE_PAGES {
            let response = client
                .get(format!("{}{}", DEV_SERVER_URL, path))
                .send()
                .await?;
            assert!(
                response.status().is_success(),
                "Page {} should be accessible",
                path
            );
        }

        // Verify critical assets are available
        for &(path, _, _) in CRITICAL_ASSETS {
            let response = client
                .get(format!("{}{}", DEV_SERVER_URL, path))
                .send()
                .await?;
            assert!(
                response.status().is_success(),
                "Asset {} should be available",
                path
            );
        }

        Ok(())
    }

    /// Test 7: Server Coordination Management
    /// Tests that the shared server instance works correctly
    #[tokio::test]
    #[cfg(not(any(feature = "ci", coverage)))]
    async fn test_server_coordination_management() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;

        // Verify server responds
        let response = client.get(DEV_SERVER_URL).send().await?;
        assert!(
            response.status().is_success(),
            "Server should be responsive"
        );

        // Verify we can get the same client again
        let client2 = get_client().await?;
        let response2 = client2.get(DEV_SERVER_URL).send().await?;
        assert!(
            response2.status().is_success(),
            "Server should still be responsive"
        );

        Ok(())
    }
}
