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
    }

    impl TestServer {
        async fn start() -> Result<Self, Box<dyn std::error::Error>> {
            let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let server_url = format!("http://127.0.0.1:{}", port);

            Self::cleanup_existing_processes(port).await;
            Self::ensure_ports_available(port).await?;

            let db_process = Self::start_database(port).await?;
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

            let db_port = 8000 + (port - 3007);
            std::env::set_var("LEPTOS_SITE_ADDR", format!("127.0.0.1:{}", port));
            std::env::set_var("SURREAL_HOST", format!("127.0.0.1:{}", db_port));

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

        async fn start_database(port: u16) -> Result<Child, Box<dyn std::error::Error>> {
            let db_port = 8000 + (port - 3007);
            let db_file = format!("rustblog_test_{}.db", port);

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

            let mut db_process = Command::new("bash")
                .arg("-c")
                .arg(&db_command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to start database: {}", e))?;

            tokio::time::sleep(Duration::from_millis(500)).await;

            if let Ok(Some(_status)) = db_process.try_wait() {
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

            Err(format!(
                "Database on port {} is not responsive within timeout",
                db_port
            )
            .into())
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

        async fn ensure_ports_available(port: u16) -> Result<(), Box<dyn std::error::Error>> {
            let reload_port = port + 1000;
            let db_port = 8000 + (port - 3007);
            let server_ports = [port, reload_port, db_port];
            let ports_in_use = server_ports
                .iter()
                .filter(|&&p| Self::is_port_in_use(p))
                .collect::<Vec<_>>();

            if !ports_in_use.is_empty() {
                return Err(format!("Server ports are in use: {:?}", ports_in_use).into());
            }
            Ok(())
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

        async fn cleanup_existing_processes(port: u16) {
            let db_port = 8000 + (port - 3007);
            let reload_port = port + 1000;
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "lsof -ti:{},{},{} | xargs -r kill -TERM 2>/dev/null || true",
                        port, reload_port, db_port
                    ),
                ])
                .output();
            tokio::time::sleep(Duration::from_millis(250)).await;
            let _ = Command::new("bash")
                .args([
                    "-c",
                    &format!(
                        "lsof -ti:{},{},{} | xargs -r kill -KILL 2>/dev/null || true",
                        port, reload_port, db_port
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
        }
    }

    async fn start_test_server() -> Result<(TestServer, String), Box<dyn std::error::Error>> {
        let server = TestServer::start().await?;
        let server_url = format!("http://127.0.0.1:{}", server.port);
        Ok((server, server_url))
    }

    #[tokio::test]
    #[ignore = "REST API endpoints not implemented yet - will be added in subsequent MR"]
    async fn test_create_and_fetch_activity() -> Result<(), Box<dyn std::error::Error>> {
        let (server, server_url) = start_test_server().await?;
        let client = server.client.clone();

        // 1. Define the activity data
        let activity_data = serde_json::json!({
            "content": "This is a test activity",
            "tags": ["test", "rust"],
            "source": "https://example.com"
        });

        // 2. Create a new activity
        let mut response = client
            .post(format!("{}/api/activities/create", server_url))
            .json(&activity_data)
            .send()
            .await?;

        // Print response details for debugging
        println!("Create activity response status: {}", response.status());
        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "No error text".to_string());
            println!("Error response: {}", error_text);
            // We need to create a new response since we consumed the old one
            response = client
                .post(format!("{}/api/activities/create", server_url))
                .json(&activity_data)
                .send()
                .await?;
        }

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
}
