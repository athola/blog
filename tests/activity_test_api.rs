#![allow(deprecated)]
use app::types::Activity;
use leptos::prelude::ServerFnError;
use leptos::server_fn::error::NoCustomError;
use std::time::Duration;
use surrealdb::Surreal;
use tokio_retry::{strategy::ExponentialBackoff, Retry};

pub type TestDb = surrealdb::engine::local::Db;

pub async fn retry_db_operation<F, Fut, T>(operation: F) -> Result<T, ServerFnError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, surrealdb::Error>>,
{
    let retry_strategy = ExponentialBackoff::from_millis(50)
        .max_delay(Duration::from_secs(2))
        .take(3);

    match Retry::spawn(retry_strategy, || async { operation().await }).await {
        Ok(result) => Ok(result),
        Err(err) => Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Database error: {}",
            err
        ))),
    }
}

pub async fn create_activity(
    db: &Surreal<TestDb>,
    activity: Activity,
) -> Result<(), ServerFnError> {
    let _created: Option<Activity> =
        retry_db_operation(|| async { db.create("activity").content(activity.clone()).await })
            .await?;

    Ok(())
}

pub async fn select_activities(
    db: &Surreal<TestDb>,
    page: usize,
) -> Result<Vec<Activity>, ServerFnError> {
    let activities_per_page = 10;
    let start = page * activities_per_page;

    let query = format!(
        "SELECT * FROM activity ORDER BY created_at DESC LIMIT {} START {};",
        activities_per_page, start
    );

    let mut query = retry_db_operation(|| async { db.query(&query).await }).await?;
    let activities = query
        .take::<Vec<Activity>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;

    Ok(activities)
}
