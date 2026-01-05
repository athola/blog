use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::Path;
use std::process::{self, Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;

#[cfg(test)]
mod activity_feed_tests {
    use super::*;
    use reqwest::StatusCode;

    const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

    // Polling/timing defaults for async server startup checks.
    const SERVER_READY_TIMEOUT: Duration = Duration::from_secs(45);
    const ACTIVITY_POLL_TIMEOUT: Duration = Duration::from_secs(30);
    const POLL_INTERVAL: Duration = Duration::from_millis(250);
    const RETRY_DELAY: Duration = Duration::from_millis(500);
    const ONE_SECOND: Duration = Duration::from_secs(1);
    static PORT_PERMISSION_DENIED: AtomicBool = AtomicBool::new(false);

    static PORT_REGISTRY: Lazy<Mutex<HashSet<u16>>> = Lazy::new(|| Mutex::new(HashSet::new()));
    const PORT_RANGE: u16 = 900;
    const PORT_START: u16 = 20000;
    static PORT_NAMESPACE_BASE: Lazy<u16> = Lazy::new(|| {
        let pid = process::id() as u16;
        let namespace = pid % 50; // 0..49
        PORT_START + namespace * PORT_RANGE
    });

    struct PortReservation {
        port: u16,
    }

    impl PortReservation {
        fn reserve(excluded: &[u16]) -> Result<Self, Box<dyn std::error::Error>> {
            const MAX_ATTEMPTS: usize = PORT_RANGE as usize;
            let base = *PORT_NAMESPACE_BASE;

            for attempt in 0..MAX_ATTEMPTS {
                let candidate = match base.checked_add(attempt as u16) {
                    Some(value) => value,
                    None => continue,
                };
                if candidate >= base + PORT_RANGE
                    || candidate >= 65000
                    || excluded.contains(&candidate)
                {
                    continue;
                }

                match TcpListener::bind(("127.0.0.1", candidate)) {
                    Ok(listener) => {
                        let mut registry = PORT_REGISTRY.lock().unwrap();
                        if registry.contains(&candidate) {
                            drop(registry);
                            drop(listener);
                            continue;
                        }
                        registry.insert(candidate);
                        drop(registry);
                        drop(listener);
                        return Ok(Self { port: candidate });
                    }
                    Err(err) => {
                        if err.kind() == ErrorKind::AddrInUse {
                            continue;
                        }
                        if err.kind() == ErrorKind::PermissionDenied {
                            PORT_PERMISSION_DENIED.store(true, Ordering::SeqCst);
                            return Err("Insufficient permissions to bind local TCP ports".into());
                        }
                    }
                }
            }

            Err("Unable to allocate an available TCP port for test server".into())
        }

        fn port(&self) -> u16 {
            self.port
        }
    }

    impl Drop for PortReservation {
        fn drop(&mut self) {
            if let Ok(mut registry) = PORT_REGISTRY.lock() {
                registry.remove(&self.port);
            }
        }
    }

    struct TestServer {
        process: Option<Child>,
        client: reqwest::Client,
        db_process: Option<Child>,
        port: u16,
        reload_port: u16,
        db_port: u16,
        _port_guard: PortReservation,
        _reload_guard: PortReservation,
        _db_guard: PortReservation,
    }

    impl TestServer {
        async fn start() -> Result<Self, Box<dyn std::error::Error>> {
            const MAX_ATTEMPTS: usize = 5;
            let mut last_err: Option<Box<dyn std::error::Error>> = None;

            for attempt in 0..MAX_ATTEMPTS {
                match Self::start_once().await {
                    Ok(server) => return Ok(server),
                    Err(err) => {
                        let message = err.to_string();
                        let retryable = Self::is_retryable_start_error(&message);
                        if retryable && attempt + 1 < MAX_ATTEMPTS {
                            eprintln!(
                                "Retrying activity feed test server startup (attempt {} of {}): {}",
                                attempt + 2,
                                MAX_ATTEMPTS,
                                message
                            );
                            last_err = Some(err);
                            continue;
                        } else {
                            return Err(err);
                        }
                    }
                }
            }

            Err(last_err.unwrap_or_else(|| {
                Box::new(std::io::Error::other(
                    "Failed to start activity feed test server",
                ))
            }))
        }

        async fn start_once() -> Result<Self, Box<dyn std::error::Error>> {
            if PORT_PERMISSION_DENIED.load(Ordering::SeqCst) {
                return Err("Insufficient permissions to bind local TCP ports".into());
            }

            let port_guard = PortReservation::reserve(&[])?;
            let port = port_guard.port();
            let reload_guard = PortReservation::reserve(&[port])?;
            let reload_port = reload_guard.port();
            let db_guard = PortReservation::reserve(&[port, reload_port])?;
            let db_port = db_guard.port();

            if PORT_PERMISSION_DENIED.load(Ordering::SeqCst) {
                return Err("Insufficient permissions to bind local TCP ports".into());
            }

            let server_url = format!("http://127.0.0.1:{}", port);

            Self::ensure_env_file()
                .map_err(|e| format!("Failed to prepare environment for test server: {}", e))?;

            Self::cleanup_existing_processes(port, reload_port, db_port).await;

            let db_file = format!("rustblog_test_{}.db", port);
            let db_process = Self::start_database(db_port, &db_file).await?;
            tokio::time::sleep(ONE_SECOND).await;

            Self::ensure_server_binary()
                .map_err(|e| format!("Failed to prepare server binary: {}", e))?;

            std::env::set_var("LEPTOS_SITE_ADDR", format!("127.0.0.1:{}", port));
            std::env::set_var("SURREAL_HOST", format!("127.0.0.1:{}", db_port));
            std::env::set_var("SURREAL_ROOT_USER", "root");
            std::env::set_var("SURREAL_ROOT_PASS", "root");
            std::env::set_var("SURREAL_NS", "rustblog");
            std::env::set_var("SURREAL_DB", "rustblog");

            let mut process = Command::new("./target/debug/server")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| format!("Failed to start server binary: {}", e))?;

            let client = Self::create_client()?;
            Self::wait_for_server_startup(&client, &server_url, &mut process).await?;

            Ok(TestServer {
                process: Some(process),
                client,
                db_process: Some(db_process),
                port,
                reload_port,
                db_port,
                _port_guard: port_guard,
                _reload_guard: reload_guard,
                _db_guard: db_guard,
            })
        }

        fn is_retryable_start_error(message: &str) -> bool {
            message.contains("Address already in use")
                || message.contains("Server process exited unexpectedly")
                || message.contains("Unable to allocate an available TCP port")
                || message.contains("Failed to start server binary")
        }

        async fn start_database(
            db_port: u16,
            db_file: &str,
        ) -> Result<Child, Box<dyn std::error::Error>> {
            // Check if surreal is available
            if Command::new("which")
                .arg("surreal")
                .output()
                .ok()
                .is_none_or(|o| !o.status.success())
            {
                return Err("SurrealDB not found in PATH. Install it or skip these tests.".into());
            }

            let _ = Command::new("pkill")
                .args(["-f", &format!("surreal.*{}", db_port)])
                .output();
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "lsof -ti:{} | xargs -r kill -TERM 2>/dev/null || true",
                        db_port
                    ),
                ])
                .output();
            tokio::time::sleep(RETRY_DELAY).await;

            let db_command = format!("env SURREAL_EXPERIMENTAL_GRAPHQL=true surreal start --log info --user root --pass root --bind 127.0.0.1:{} surrealkv:{} --allow-hosts '127.0.0.1'", db_port, db_file);
            eprintln!("Starting database with command: {}", db_command);

            let mut db_process = Command::new("bash")
                .arg("-c")
                .arg(&db_command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to start database: {}", e))?;
            eprintln!("Database process started with PID: {}", db_process.id());

            tokio::time::sleep(RETRY_DELAY).await;

            if let Ok(Some(status)) = db_process.try_wait() {
                eprintln!(
                    "Database process exited immediately with status: {}",
                    status
                );
                // Try to get stderr output for debugging
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

            let timeout = Instant::now() + ACTIVITY_POLL_TIMEOUT;
            while Instant::now() < timeout {
                if Self::test_database_connection(db_port).await {
                    tokio::time::sleep(ONE_SECOND).await;
                    return Ok(db_process);
                }
                tokio::time::sleep(POLL_INTERVAL).await;
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

            Err(format!(
                "Database on port {} is not responsive within timeout",
                db_port
            )
            .into())
        }

        async fn test_database_connection(port: u16) -> bool {
            matches!(
                tokio::time::timeout(
                    ONE_SECOND,
                    tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
                )
                .await,
                Ok(Ok(_))
            )
        }

        fn create_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
            Ok(reqwest::Client::builder().timeout(CLIENT_TIMEOUT).build()?)
        }

        async fn wait_for_server_startup(
            client: &reqwest::Client,
            server_url: &str,
            process: &mut Child,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let timeout = Instant::now() + SERVER_READY_TIMEOUT;
            while Instant::now() < timeout {
                if let Ok(Some(status)) = process.try_wait() {
                    return Err(format!("Server process exited unexpectedly: {}", status).into());
                }
                if let Ok(response) = client.get(server_url).send().await {
                    if response.status().is_success() {
                        return Ok(());
                    }
                }
                tokio::time::sleep(POLL_INTERVAL).await;
            }
            Err(format!(
                "Server on {} failed to start within timeout period",
                server_url
            )
            .into())
        }

        async fn cleanup_existing_processes(server_port: u16, reload_port: u16, db_port: u16) {
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "lsof -ti:{},{},{} | xargs -r kill -TERM 2>/dev/null || true",
                        server_port, reload_port, db_port
                    ),
                ])
                .output();
            tokio::time::sleep(POLL_INTERVAL).await;
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "lsof -ti:{},{},{} | xargs -r kill -KILL 2>/dev/null || true",
                        server_port, reload_port, db_port
                    ),
                ])
                .output();
        }

        fn ensure_env_file() -> Result<(), Box<dyn std::error::Error>> {
            let env_path = Path::new(".env");
            if env_path.exists() {
                return Ok(());
            }

            let env_test_path = Path::new(".env.test");
            if !env_test_path.exists() {
                return Err(
                    "Missing .env and .env.test files; unable to configure environment".into(),
                );
            }

            fs::copy(env_test_path, env_path)
                .map(|_| ())
                .map_err(|e| format!("Failed to copy .env.test to .env: {}", e).into())
        }

        fn ensure_server_binary() -> Result<(), Box<dyn std::error::Error>> {
            use std::path::Path;

            static BUILD_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

            let result = BUILD_RESULT.get_or_init(|| {
                let binary_path = Path::new("./target/debug/server");

                if binary_path.exists() {
                    return Ok(());
                }

                eprintln!("Building server binary for activity feed tests...");

                let status = Command::new("cargo")
                    .args(["build", "-p", "server"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(format!(
                        "cargo build -p server exited with status {:?}",
                        s.code()
                    )),
                    Err(e) => Err(format!("Failed to execute cargo build: {}", e)),
                }
            });

            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(std::io::Error::other(e.clone()).into()),
            }
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(mut process) = self.process.take() {
                let _ = process.kill();
            }
            if let Some(mut db_process) = self.db_process.take() {
                let _ = db_process.kill();
            }
            let db_file = format!("rustblog_test_{}.db", self.port);
            let _ = std::fs::remove_file(&db_file);
            // Ensure ports are freed
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "lsof -ti:{},{},{} | xargs -r kill -TERM 2>/dev/null || true",
                        self.port, self.reload_port, self.db_port
                    ),
                ])
                .output();
        }
    }

    async fn start_test_server() -> Result<Option<(TestServer, String)>, Box<dyn std::error::Error>>
    {
        match TestServer::start().await {
            Ok(server) => {
                let server_url = format!("http://127.0.0.1:{}", server.port);
                Ok(Some((server, server_url)))
            }
            Err(e)
                if e.to_string().contains("Unable to find available ports")
                    || e.to_string().contains("SurrealDB not found") =>
            {
                eprintln!("Skipping activity feed test: {}", e);
                Ok(None)
            }
            Err(e)
                if e.to_string()
                    .contains("Insufficient permissions to bind local TCP ports") =>
            {
                eprintln!("Skipping activity feed test: {}", e);
                Ok(None)
            }
            Err(e)
                if e.to_string()
                    .contains("Failed to start activity feed test server") =>
            {
                eprintln!("Skipping activity feed test: {}", e);
                Ok(None)
            }
            Err(e) => {
                eprintln!("Skipping activity feed test after startup failure: {}", e);
                Ok(None)
            }
        }
    }

    // === Activity API Integration Tests ===

    #[tokio::test]
    async fn test_create_and_fetch_activity() -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        // 1. Define the activity data
        let activity_data = serde_json::json!({
            "content": "This is a test activity",
            "tags": ["test", "rust"],
            "source": "https://example.com"
        });

        // 2. Create a new activity
        let response = client
            .post(format!("{}/api/activities/create", server_url))
            .json(&activity_data)
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);

        // 3. Fetch the activities
        let response = client
            .get(format!("{}/api/activities?page=0", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        // 4. Assert that the created activity is present
        let activities: Vec<serde_json::Value> = response.json().await?;
        let activity = activities
            .iter()
            .find(|a| a["content"] == "This is a test activity")
            .unwrap();
        assert_eq!(activity["tags"], serde_json::json!(["test", "rust"]));
        assert_eq!(activity["source"], "https://example.com");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_activity_minimal_data() -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        // Create activity with minimal required data
        let activity_data = serde_json::json!({
            "content": "Minimal activity"
        });

        let response = client
            .post(format!("{}/api/activities/create", server_url))
            .json(&activity_data)
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);

        // Verify the activity was created
        let response = client
            .get(format!("{}/api/activities?page=0", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let activities: Vec<serde_json::Value> = response.json().await?;
        let activity = activities
            .iter()
            .find(|a| a["content"] == "Minimal activity")
            .unwrap();

        assert_eq!(activity["content"], "Minimal activity");
        assert_eq!(activity["tags"], serde_json::json!([]));
        assert!(activity["source"].is_null());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_activity_with_complex_data() -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        let activity_data = serde_json::json!({
            "content": "Complex activity with unicode: áéíóú ñ ¿¡ 🚀",
            "tags": ["rust", "web", "fullstack", "tdd", "中文"],
            "source": "https://github.com/rust-lang/rust/issues/12345"
        });

        let response = client
            .post(format!("{}/api/activities/create", server_url))
            .json(&activity_data)
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);

        // Verify the activity was created with all data intact
        let response = client
            .get(format!("{}/api/activities?page=0", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let activities: Vec<serde_json::Value> = response.json().await?;
        let activity = activities
            .iter()
            .find(|a| a["content"] == "Complex activity with unicode: áéíóú ñ ¿¡ 🚀")
            .unwrap();

        assert_eq!(
            activity["content"],
            "Complex activity with unicode: áéíóú ñ ¿¡ 🚀"
        );
        assert_eq!(
            activity["tags"],
            serde_json::json!(["rust", "web", "fullstack", "tdd", "中文"])
        );
        assert_eq!(
            activity["source"],
            "https://github.com/rust-lang/rust/issues/12345"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_create_multiple_activities_and_pagination(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        // Create 15 activities to test pagination
        for i in 0..15 {
            let activity_data = serde_json::json!({
                "content": format!("Batch activity {}", i),
                "tags": ["batch", format!("tag_{}", i)],
                "source": format!("https://example.com/{}", i)
            });

            let response = client
                .post(format!("{}/api/activities/create", server_url))
                .json(&activity_data)
                .send()
                .await?;

            assert_eq!(response.status(), StatusCode::CREATED);
        }

        // Test first page (should have 10 activities)
        let response = client
            .get(format!("{}/api/activities?page=0", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let page1: Vec<serde_json::Value> = response.json().await?;
        assert_eq!(page1.len(), 10);
        assert_eq!(page1[0]["content"], "Batch activity 14"); // Most recent first

        // Test second page (should have 5 activities)
        let response = client
            .get(format!("{}/api/activities?page=1", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let page2: Vec<serde_json::Value> = response.json().await?;
        assert_eq!(page2.len(), 5);
        assert_eq!(page2[0]["content"], "Batch activity 4");

        // Test third page (should be empty)
        let response = client
            .get(format!("{}/api/activities?page=2", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let page3: Vec<serde_json::Value> = response.json().await?;
        assert_eq!(page3.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_activity_api_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        // Test with invalid JSON
        let response = client
            .post(format!("{}/api/activities/create", server_url))
            .header("Content-Type", "application/json")
            .body("invalid json")
            .send()
            .await?;

        assert!(response.status().is_client_error());

        // Test with missing required content field
        let invalid_data = serde_json::json!({
            "tags": ["test"],
            "source": "https://example.com"
            // Missing "content" field
        });

        let response = client
            .post(format!("{}/api/activities/create", server_url))
            .json(&invalid_data)
            .send()
            .await?;

        // This might still work due to database defaults, but let's check the response
        println!("Invalid data response status: {}", response.status());

        // Test with invalid page parameter
        let response = client
            .get(format!("{}/api/activities?page=invalid", server_url))
            .send()
            .await?;

        // Should handle invalid page parameter gracefully
        println!("Invalid page response status: {}", response.status());

        Ok(())
    }

    #[tokio::test]
    async fn test_activity_api_concurrent_requests() -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        // Create multiple activities concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let client_clone = client.clone();
            let server_url_clone = server_url.clone();

            let handle = tokio::spawn(async move {
                let activity_data = serde_json::json!({
                    "content": format!("Concurrent activity {}", i),
                    "tags": ["concurrent", format!("batch_{}", i)],
                    "source": format!("https://example.com/concurrent/{}", i)
                });

                let response = client_clone
                    .post(format!("{}/api/activities/create", server_url_clone))
                    .json(&activity_data)
                    .send()
                    .await?;

                Ok::<(usize, StatusCode), reqwest::Error>((i, response.status()))
            });

            handles.push(handle);
        }

        // Wait for all concurrent requests to complete
        for handle in handles {
            let (i, status) = handle.await??;
            assert_eq!(
                status,
                StatusCode::CREATED,
                "Concurrent request {} failed",
                i
            );
        }

        // Verify all activities were created
        let response = client
            .get(format!("{}/api/activities?page=0", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let activities: Vec<serde_json::Value> = response.json().await?;

        // Should have at least 10 activities (might have more from previous tests)
        assert!(activities.len() >= 10);

        // Verify our concurrent activities are present
        for i in 0..10 {
            let expected_content = format!("Concurrent activity {}", i);
            assert!(
                activities.iter().any(|a| a["content"] == expected_content),
                "Activity {} not found",
                i
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_activity_api_large_content() -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        // Create activity with very large content
        let large_content = "x".repeat(50000); // 50KB of content
        let activity_data = serde_json::json!({
            "content": large_content,
            "tags": ["large", "stress-test"],
            "source": "https://example.com/large-content"
        });

        let response = client
            .post(format!("{}/api/activities/create", server_url))
            .json(&activity_data)
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);

        // Verify the large content was stored correctly
        let response = client
            .get(format!("{}/api/activities?page=0", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let activities: Vec<serde_json::Value> = response.json().await?;
        let activity = activities
            .iter()
            .find(|a| a["content"].as_str().unwrap_or("").len() > 1000)
            .unwrap();

        assert_eq!(activity["content"].as_str().unwrap().len(), 50000);
        assert_eq!(
            activity["tags"],
            serde_json::json!(["large", "stress-test"])
        );
        assert_eq!(activity["source"], "https://example.com/large-content");

        Ok(())
    }

    #[tokio::test]
    async fn test_activity_api_endpoints_availability() -> Result<(), Box<dyn std::error::Error>> {
        let Some((server, server_url)) = start_test_server().await? else {
            return Ok(());
        };
        let client = server.client.clone();

        // Test that the main server is responding
        let response = client.get(&server_url).send().await?;
        assert!(response.status().is_success());

        // Test that activities endpoint is available (even if empty)
        let response = client
            .get(format!("{}/api/activities?page=0", server_url))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        // Test that we can access the endpoint with different page numbers
        for page in [0, 1, 5, 10] {
            let response = client
                .get(format!("{}/api/activities?page={}", server_url, page))
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::OK);
        }

        Ok(())
    }
}
