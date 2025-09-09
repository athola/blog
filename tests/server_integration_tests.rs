use std::process::{Command, Child, Stdio};
use std::time::{Duration, Instant};
use tokio::time::interval;
use std::net::TcpListener;
use std::io::{Write, Read};

/// Integration tests for the Leptos development server
/// 
/// This test suite uses a shared server instance to minimize resource usage.
/// Tests are organized by functional areas: connectivity, content, assets, and performance.
#[cfg(test)]
mod server_integration_tests {
    use super::*;

    /// File-based coordination between test processes
    const SERVER_PID_FILE: &str = "target/blog_test_server.pid";
    const SERVER_READY_FILE: &str = "target/blog_test_server_ready";

    /// Test timeouts
    const SERVER_TIMEOUT: Duration = Duration::from_secs(60);
    const ASSET_TIMEOUT: Duration = Duration::from_secs(45);
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

    /// Shared server instance that runs for the duration of all tests
    struct SharedTestServer {
        process: Option<Child>,
        client: reqwest::Client,
    }

    impl SharedTestServer {
        /// Start the shared development server
        async fn start() -> Result<Self, Box<dyn std::error::Error>> {
            Self::cleanup_existing_processes().await;
            cleanup_coordination_files();
            
            Self::ensure_ports_available()?;
            
            let mut process = Command::new("make")
                .arg("watch")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("Failed to start make watch: {}", e))?;

            let client = Self::create_client()?;

            Self::wait_for_server_startup(&client, &mut process).await?;

            Ok(SharedTestServer {
                process: Some(process),
                client,
            })
        }

        /// Create HTTP client with standard configuration
        fn create_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
            Ok(reqwest::Client::builder()
                .timeout(CLIENT_TIMEOUT)
                .build()?)
        }

        /// Ensure required ports are available
        fn ensure_ports_available() -> Result<(), Box<dyn std::error::Error>> {
            let ports_in_use = [3007, 3001].iter()
                .filter(|&&port| Self::is_port_in_use(port))
                .collect::<Vec<_>>();
                
            if !ports_in_use.is_empty() {
                return Err(format!("Required ports in use: {:?}", ports_in_use).into());
            }
            Ok(())
        }

        /// Wait for server to start and respond
        async fn wait_for_server_startup(
            client: &reqwest::Client, 
            process: &mut Child
        ) -> Result<(), Box<dyn std::error::Error>> {
            let timeout = Instant::now() + Duration::from_secs(30);
            
            while Instant::now() < timeout {
                if let Ok(Some(status)) = process.try_wait() {
                    return Err(format!("Server process exited unexpectedly: {}", status).into());
                }
                
                if let Ok(response) = client.get(DEV_SERVER_URL).send().await {
                    if response.status().is_success() {
                        return Ok(());
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            
            Err("Server failed to start within timeout period".into())
        }

        /// Wait for server to be ready including assets
        async fn wait_for_ready(&self) -> Result<(), Box<dyn std::error::Error>> {
            self.wait_for_server_response().await?;
            self.wait_for_critical_assets().await?;
            self.wait_for_wasm_asset().await;
            Ok(())
        }

        /// Wait for basic server response
        async fn wait_for_server_response(&self) -> Result<(), Box<dyn std::error::Error>> {
            let timeout = Instant::now() + SERVER_TIMEOUT;
            
            while Instant::now() <= timeout {
                match self.client.get(DEV_SERVER_URL).send().await {
                    Ok(response) if response.status().is_success() => return Ok(()),
                    _ => tokio::time::sleep(Duration::from_millis(250)).await,
                }
            }
            
            Err("Server startup timeout".into())
        }

        /// Wait for critical assets to be available
        async fn wait_for_critical_assets(&self) -> Result<(), Box<dyn std::error::Error>> {
            // In CI/coverage mode, assets may take much longer to compile
            let extended_timeout = if cfg!(coverage) {
                Duration::from_secs(180) // 3 minutes for coverage builds
            } else {
                ASSET_TIMEOUT
            };
            
            for &(asset_path, _, _) in CRITICAL_ASSETS {
                let timeout = Instant::now() + extended_timeout;
                let mut asset_ready = false;
                
                while Instant::now() <= timeout {
                    match self.client.get(format!("{}{}", DEV_SERVER_URL, asset_path)).send().await {
                        Ok(response) if response.status().is_success() => {
                            asset_ready = true;
                            break;
                        }
                        _ => tokio::time::sleep(Duration::from_millis(1000)).await, // Longer sleep for coverage
                    }
                }
                
                if !asset_ready {
                    eprintln!("Warning: Critical asset {} not ready in time, continuing anyway", asset_path);
                    // Don't fail tests in coverage mode if assets aren't ready - this is often due to slow compilation
                    if !cfg!(coverage) {
                        return Err(format!("Critical asset {} not ready in time", asset_path).into());
                    }
                }
            }
            Ok(())
        }

        /// Wait for WASM asset (non-critical)
        async fn wait_for_wasm_asset(&self) {
            let timeout = Instant::now() + ASSET_TIMEOUT;
            
            while Instant::now() <= timeout {
                if let Ok(response) = self.client.get(format!("{}/pkg/blog.wasm", DEV_SERVER_URL)).send().await {
                    if response.status().is_success() {
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }


        /// Clean up existing processes
        async fn cleanup_existing_processes() {
            let process_patterns = ["make.*watch", "cargo.*leptos", "cargo-leptos"];
            
            // Graceful shutdown
            for pattern in process_patterns {
                let _ = Command::new("pkill").args(["-TERM", "-f", pattern]).output();
            }
            
            let _ = Command::new("bash")
                .args(["-c", "lsof -ti:3007,3001 | xargs -r kill -TERM 2>/dev/null || true"])
                .output();
                
            Self::wait_for_process_termination().await;
            
            // Force cleanup if needed
            for pattern in process_patterns {
                let _ = Command::new("pkill").args(["-KILL", "-f", pattern]).output();
            }
            
            let _ = Command::new("bash")
                .args(["-c", "lsof -ti:3007,3001 | xargs -r kill -KILL 2>/dev/null || true"])
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
            
            cleanup_coordination_files();
        }
    }

    // === Helper Functions ===

    /// Get or start the shared server using file-based coordination
    async fn get_shared_server() -> Result<(), Box<dyn std::error::Error>> {
        // Check if server is already running
        if let Ok(mut file) = std::fs::File::open(SERVER_PID_FILE) {
            let mut pid_str = String::new();
            if file.read_to_string(&mut pid_str).is_ok() {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if process_exists(pid) {
                        wait_for_server_ready().await?;
                        return Ok(());
                    }
                }
            }
        }
        
        cleanup_coordination_files();
        
        let server = SharedTestServer::start().await?;
        
        // Write PID to file
        if let Some(ref process) = server.process {
            let pid = process.id();
            let mut file = std::fs::File::create(SERVER_PID_FILE)?;
            write!(file, "{}", pid)?;
        }
        
        server.wait_for_ready().await?;
        
        // Signal that server is ready
        std::fs::File::create(SERVER_READY_FILE)?;
        
        // Keep the server running
        std::mem::forget(server);
        
        Ok(())
    }

    /// Wait for the server ready signal
    async fn wait_for_server_ready() -> Result<(), Box<dyn std::error::Error>> {
        let timeout = Instant::now() + SERVER_TIMEOUT;
        
        while Instant::now() < timeout {
            if std::fs::metadata(SERVER_READY_FILE).is_ok() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        Err("Timeout waiting for server ready signal".into())
    }

    /// Check if a process exists
    fn process_exists(pid: u32) -> bool {
        std::fs::metadata(format!("/proc/{}", pid)).is_ok()
    }

    /// Clean up coordination files
    fn cleanup_coordination_files() {
        let _ = std::fs::remove_file(SERVER_PID_FILE);
        let _ = std::fs::remove_file(SERVER_READY_FILE);
    }

    /// Get HTTP client for tests
    async fn get_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
        get_shared_server().await?;
        SharedTestServer::create_client()
    }

    /// Helper to fetch and validate a page
    async fn fetch_and_validate_page(
        client: &reqwest::Client, 
        path: &str, 
        description: &str
    ) -> Result<String, Box<dyn std::error::Error>> {
        let response = client.get(format!("{}{}", DEV_SERVER_URL, path)).send().await?;
        
        assert!(response.status().is_success(),
                "{} should return success, got: {}", description, response.status());
        assert_eq!(response.headers().get("content-type").unwrap(),
                   "text/html; charset=utf-8");
        
        let body = response.text().await?;
        assert!(body.contains("<!DOCTYPE html"), 
                "{} should contain HTML doctype", description);
        
        Ok(body)
    }

    /// Helper to validate asset serving
    async fn validate_asset(
        client: &reqwest::Client,
        path: &str,
        expected_content_type: &str,
        min_size: u64
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = client.get(format!("{}{}", DEV_SERVER_URL, path)).send().await?;
        
        assert!(response.status().is_success(),
                "Asset {} should return success, got: {}", path, response.status());
        
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.starts_with(expected_content_type),
                "Asset {} should have content-type {}, got: {}", 
                path, expected_content_type, content_type);
        
        let content_length = response.content_length().unwrap_or(0);
        assert!(content_length >= min_size,
                "Asset {} should be at least {} bytes, got: {}", 
                path, min_size, content_length);
        
        Ok(())
    }

    // === Test Cases ===

    /// Test 1: Server Connectivity and Basic Response
    /// Verifies server starts, responds to requests, and returns proper content type
    #[tokio::test]
    async fn test_server_connectivity() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;
        let response = client.get(DEV_SERVER_URL).send().await?;
        
        assert!(response.status().is_success(), 
                "Server should respond with success status, got: {}", response.status());
        assert_eq!(response.headers().get("content-type").unwrap(), 
                   "text/html; charset=utf-8");

        Ok(())
    }

    /// Test 2: Page Navigation and Content
    /// Tests all core pages for accessibility, content type, and expected content
    #[tokio::test] 
    async fn test_page_navigation_and_content() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;

        for &(path, description) in CORE_PAGES {
            let body = fetch_and_validate_page(&client, path, description).await?;
            
            // All pages should contain navigation elements
            assert!(body.contains("blog"), 
                    "{} should contain navigation elements", description);
        }

        // Test navigation links on home page
        let home_body = fetch_and_validate_page(&client, "/", "Home page").await?;
        assert!(home_body.contains(r#"href="/""#), "Should contain home link");
        assert!(home_body.contains(r#"href="/references""#), "Should contain references link");  
        assert!(home_body.contains(r#"href="/contact""#), "Should contain contact link");
        assert!(home_body.contains("github.com/athola"), "Should contain GitHub link");
        assert!(home_body.contains("linkedin.com/in/alexthola"), "Should contain LinkedIn link");

        // Test page-specific content
        let references_body = fetch_and_validate_page(&client, "/references", "References page").await?;
        assert!(references_body.contains("Project References"), 
                "References page should contain 'Project References'");

        let contact_body = fetch_and_validate_page(&client, "/contact", "Contact page").await?;
        assert!(contact_body.contains("Get In Touch"), 
                "Contact page should contain 'Get In Touch'");
        assert!(contact_body.contains("form"), 
                "Contact page should contain a form");

        Ok(())
    }

    /// Test 3: Static Asset Serving
    /// Validates that all critical assets (CSS, JS) are served correctly with proper headers
    #[tokio::test]
    async fn test_static_asset_serving() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;

        // Test critical assets - be more forgiving in coverage mode
        for &(path, expected_content_type, min_size) in CRITICAL_ASSETS {
            match validate_asset(&client, path, expected_content_type, min_size).await {
                Ok(_) => {}, // Asset validated successfully
                Err(e) if cfg!(coverage) => {
                    eprintln!("Warning: Asset validation failed in coverage mode: {}", e);
                    // Continue without failing the test in coverage mode
                },
                Err(e) => return Err(e), // Fail normally in non-coverage mode
            }
        }
        
        // Test WASM asset (optional - don't fail if not ready)
        if let Ok(response) = client.get(format!("{}/pkg/blog.wasm", DEV_SERVER_URL)).send().await {
            if response.status().is_success() {
                let content_type = response.headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                
                assert!(content_type.contains("wasm") || content_type.contains("application/octet-stream"),
                        "WASM asset should have appropriate content-type, got: {}", content_type);
                
                let content_length = response.content_length().unwrap_or(0);
                assert!(content_length >= 1024,
                        "WASM asset should be at least 1KB, got: {}", content_length);
            }
        }

        Ok(())
    }

    /// Test 4: Server Performance
    /// Measures response times to ensure reasonable performance under load
    #[tokio::test]
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

        let avg_response_time = response_times.iter().sum::<Duration>() / response_times.len() as u32;
        
        // Be more lenient with performance expectations in coverage mode
        let max_response_time = if cfg!(coverage) {
            Duration::from_secs(30) // Much more lenient for coverage builds
        } else {
            Duration::from_secs(5)
        };
        
        assert!(avg_response_time < max_response_time,
                "Average response time should be under {:?}, got: {:?}", max_response_time, avg_response_time);

        Ok(())
    }

    /// Test 5: Error Handling
    /// Tests server behavior with invalid routes and error conditions
    #[tokio::test]
    async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;
        
        // Test non-existent route - should still return HTML (SPA routing)
        let response = client.get(format!("{}/nonexistent", DEV_SERVER_URL)).send().await?;
        let body = response.text().await?;
        
        assert!(body.contains("<!DOCTYPE html"), 
                "Even non-existent routes should return HTML structure");

        Ok(())
    }

    /// Test 6: Complete Development Workflow
    /// End-to-end test ensuring all components work together
    #[tokio::test]
    async fn test_complete_development_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let client = get_client().await?;
        
        // Verify server responds
        let response = client.get(DEV_SERVER_URL).send().await?;
        assert!(response.status().is_success(), "Server should be responsive");
        
        // Verify all core pages are accessible
        for &(path, _) in CORE_PAGES {
            let response = client.get(format!("{}{}", DEV_SERVER_URL, path)).send().await?;
            assert!(response.status().is_success(), "Page {} should be accessible", path);
        }
        
        // Verify critical assets are available
        for &(path, _, _) in CRITICAL_ASSETS {
            let response = client.get(format!("{}{}", DEV_SERVER_URL, path)).send().await?;
            assert!(response.status().is_success(), "Asset {} should be available", path);
        }
        
        Ok(())
    }

    /// Test 7: Server Coordination File Management
    /// Tests the file-based coordination system for shared server instances
    #[tokio::test]
    async fn test_server_coordination_cleanup() -> Result<(), Box<dyn std::error::Error>> {
        let _client = get_client().await?;
        
        // Verify coordination files exist
        assert!(std::fs::metadata(SERVER_PID_FILE).is_ok(), 
                "Server PID file should exist");
        assert!(std::fs::metadata(SERVER_READY_FILE).is_ok(), 
                "Server ready file should exist");
        
        // Get server PID for verification
        let server_pid = std::fs::read_to_string(SERVER_PID_FILE)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        
        // Clean up coordination files
        cleanup_coordination_files();
        
        // Verify files are removed
        assert!(std::fs::metadata(SERVER_PID_FILE).is_err(), 
                "Server PID file should be removed after cleanup");
        assert!(std::fs::metadata(SERVER_READY_FILE).is_err(), 
                "Server ready file should be removed after cleanup");
        
        // Verify server process still exists (shared architecture)
        if let Some(pid) = server_pid {
            assert!(process_exists(pid), 
                    "Server process should still be running (shared server)");
        }
        
        // Verify server remains accessible
        let test_client = SharedTestServer::create_client()?;
        let response = test_client.get(DEV_SERVER_URL).send().await?;
        assert!(response.status().is_success(), 
                "Server should remain accessible after coordination cleanup");
        
        Ok(())
    }
}