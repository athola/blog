use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{Duration, Instant};

#[cfg(test)]
mod activity_feed_tests {
    use super::*;
    use reqwest::StatusCode;

    const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(3030);

    struct TestServer {
        process: Option<Child>,
        client: reqwest::Client,
        db_process: Option<Child>,
        port: u16,
        reload_port: u16,
        db_port: u16,
    }

    impl TestServer {
        async fn start() -> Result<Self, Box<dyn std::error::Error>> {
            let mut attempts = 0;
            let port = loop {
                let candidate = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
                let reload_candidate = candidate.saturating_add(1000);
                if Self::is_port_in_use(candidate) || Self::is_port_in_use(reload_candidate) {
                    attempts += 1;
                    if attempts > 256 {
                        return Err("Unable to find available ports for test server".into());
                    }
                    continue;
                }
                break candidate;
            };

            let server_url = format!("http://127.0.0.1:{}", port);
            let reload_port = port + 1000;
            let db_port = Self::find_available_db_port(port, reload_port)?;

            Self::cleanup_existing_processes(port, reload_port, db_port).await;

            let db_file = format!("rustblog_test_{}.db", port);
            let db_process = Self::start_database(db_port, &db_file).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;

            let build_status = Command::new("cargo")
                .args(["build", "-p", "server"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|e| format!("Failed to build server: {}", e))?;

            if !build_status.success() {
                return Err("Failed to build server".into());
            }

            std::env::set_var("LEPTOS_SITE_ADDR", format!("127.0.0.1:{}", port));
            std::env::set_var("SURREAL_HOST", format!("127.0.0.1:{}", db_port));

            let mut process = Command::new("./target/debug/server")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
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
                reload_port,
                db_port,
            })
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
            tokio::time::sleep(Duration::from_millis(500)).await;

            let db_command = format!("env SURREAL_EXPERIMENTAL_GRAPHQL=true surreal start --log info --user root --pass root --bind 127.0.0.1:{} surrealkv:{}", db_port, db_file);
            eprintln!("Starting database with command: {}", db_command);

            let mut db_process = Command::new("bash")
                .arg("-c")
                .arg(&db_command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to start database: {}", e))?;
            eprintln!("Database process started with PID: {}", db_process.id());

            tokio::time::sleep(Duration::from_millis(500)).await;

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

            let timeout = Instant::now() + Duration::from_secs(30);
            while Instant::now() < timeout {
                if Self::test_database_connection(db_port).await {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    return Ok(db_process);
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
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

        fn find_available_db_port(
            server_port: u16,
            reload_port: u16,
        ) -> Result<u16, Box<dyn std::error::Error>> {
            for port in 9000..10000 {
                if port != server_port && port != reload_port && !Self::is_port_in_use(port) {
                    return Ok(port);
                }
            }
            Err("Unable to find available database port".into())
        }

        async fn test_database_connection(port: u16) -> bool {
            matches!(
                tokio::time::timeout(
                    Duration::from_secs(1),
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
            let timeout = Instant::now() + Duration::from_secs(45);
            while Instant::now() < timeout {
                if let Ok(Some(status)) = process.try_wait() {
                    return Err(format!("Server process exited unexpectedly: {}", status).into());
                }
                if let Ok(response) = client.get(server_url).send().await {
                    if response.status().is_success() {
                        return Ok(());
                    }
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
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
            tokio::time::sleep(Duration::from_millis(250)).await;
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

        fn is_port_in_use(port: u16) -> bool {
            TcpListener::bind(("127.0.0.1", port)).is_err()
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
            Err(e) => Err(e),
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
