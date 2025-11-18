#![allow(deprecated)]
use app::types::Activity;
use leptos::prelude::ServerFnError;
use leptos::server_fn::error::NoCustomError;
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb_types::{RecordId, RecordIdKey};
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
    let _created: Option<Activity> = retry_db_operation(|| {
        let activity = activity.clone();
        async move { create_or_insert_activity(db, activity).await }
    })
    .await?;

    Ok(())
}

async fn create_or_insert_activity(
    db: &Surreal<TestDb>,
    mut activity: Activity,
) -> Result<Option<Activity>, surrealdb::Error> {
    if let Some(id) = activity.id.take() {
        Ok(Some(
            create_activity_with_fixed_id(db, &id, activity).await?,
        ))
    } else {
        db.create("activity").content(activity).await
    }
}

async fn create_activity_with_fixed_id(
    db: &Surreal<TestDb>,
    id: &RecordId,
    mut activity: Activity,
) -> Result<Activity, surrealdb::Error> {
    activity.id = None;

    let query = build_create_query(id, &activity);
    let mut response = db.query(query).await?;

    response
        .take(0)
        .map_err(|e| surrealdb::Error::Query(e.to_string()))
        .map(|opt: Option<Activity>| opt.expect("CREATE should return a record"))
}

fn build_create_query(id: &RecordId, activity: &Activity) -> String {
    let table = id.table.as_str();
    let key = record_key_literal(&id.key);
    let payload = serde_json::to_string(activity).unwrap();
    format!("CREATE {table}:{key} CONTENT {payload} RETURN *")
}

fn record_key_literal(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(value) => value.clone(),
        RecordIdKey::Number(value) => value.to_string(),
        other => panic!("Unsupported record id key variant in tests: {:?}", other),
    }
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
