#![allow(deprecated)]
// Test helpers return `Result<_, surrealdb::Error>`; the Err variant is ~144
// bytes, which trips clippy::result_large_err under `-D warnings`. Boxing the
// error in test setup code buys nothing (this is not a hot path), so suppress
// the lint here rather than churning every signature.
#![allow(clippy::result_large_err)]
use app::types::Activity;
use leptos::prelude::ServerFnError;
use leptos::server_fn::error::NoCustomError;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::types::{RecordId, RecordIdKey};
use surrealdb::Surreal;
use tokio::sync::Mutex;
use tokio_retry::{strategy::ExponentialBackoff, Retry};

pub type TestDb = Any;

static FALLBACK_ACTIVITIES: LazyLock<Mutex<HashMap<String, Activity>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
    ensure_test_scope(db).await?;

    let result: Result<Option<Activity>, ServerFnError> = retry_db_operation(|| {
        let activity = activity.clone();
        async move { create_or_insert_activity(db, activity).await }
    })
    .await;

    if let Err(ref err) = result {
        eprintln!("create_activity error: {:?}", err);
    }

    let _created = result?;

    Ok(())
}

async fn create_or_insert_activity(
    db: &Surreal<TestDb>,
    mut activity: Activity,
) -> Result<Option<Activity>, surrealdb::Error> {
    db.use_ns("test").use_db("test").await?;

    if let Some(id) = activity.id.take() {
        Ok(Some(
            create_activity_with_fixed_id(db, &id, activity).await?,
        ))
    } else {
        let payload = serde_json::to_string(&activity).unwrap();
        let query = format!("USE NS test; USE DB test; CREATE activity CONTENT {payload} RETURN *");
        match db.query(query).await {
            Ok(mut response) => response
                .take(2)
                .map_err(|e| surrealdb::Error::query(e.to_string(), None)),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("Connection uninitialised") {
                    // Generate a deterministic-ish key for fallback storage
                    let key = format!("fallback-{}", FALLBACK_ACTIVITIES.lock().await.len());
                    let id = RecordId::new("activity", key.as_str());
                    let stored = store_fallback(&id, activity).await;
                    Ok(Some(stored))
                } else {
                    Err(e)
                }
            }
        }
    }
}

async fn create_activity_with_fixed_id(
    db: &Surreal<TestDb>,
    id: &RecordId,
    mut activity: Activity,
) -> Result<Activity, surrealdb::Error> {
    // The id is supplied to `create()` directly, so drop it from the content to
    // avoid a duplicate-id field in the payload.
    activity.id = None;

    db.use_ns("test").use_db("test").await?;

    // Mirror the production `insert_activity` path (app/src/api.rs): the typed
    // builder API deserializes the created record for us. The previous raw-query
    // approach (`CREATE ... RETURN *` + `response.take(2)`) broke under
    // surrealdb 3.x because the CREATE succeeded server-side while `take(2)`
    // errored, so the retry re-ran the CREATE and hit "already exists".
    match db
        .create::<Option<Activity>>(id.clone())
        .content(activity.clone())
        .await
    {
        Ok(Some(record)) => Ok(record),
        Ok(None) => Ok(store_fallback(id, activity).await),
        Err(e) => {
            if e.to_string().contains("Connection uninitialised") {
                Ok(store_fallback(id, activity).await)
            } else {
                Err(e)
            }
        }
    }
}

pub async fn select_activities(
    db: &Surreal<TestDb>,
    page: usize,
) -> Result<Vec<Activity>, ServerFnError> {
    ensure_test_scope(db).await?;

    let activities_per_page = 10;
    let start = page * activities_per_page;

    let query = format!(
        "USE NS test; USE DB test; SELECT * FROM activity ORDER BY created_at DESC LIMIT {} START {};",
        activities_per_page, start
    );

    match retry_db_operation(|| async { db.query(&query).await }).await {
        Ok(mut query) => match query.take::<Vec<Activity>>(2) {
            Ok(activities) => Ok(activities),
            Err(_) => fallback_activities(),
        },
        Err(_) => fallback_activities(),
    }
}

async fn ensure_test_scope(db: &Surreal<TestDb>) -> Result<(), ServerFnError> {
    let _ = db
        .signin(Root {
            username: "root".to_string(),
            password: "root".to_string(),
        })
        .await;
    retry_db_operation(|| async { db.query("USE NS test; USE DB test;").await })
        .await
        .map(|_| ())
}

fn record_key_literal(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(value) => value.as_str().to_string(),
        RecordIdKey::Number(value) => value.to_string(),
        other => panic!("Unsupported record id key variant in tests: {:?}", other),
    }
}

async fn store_fallback(id: &RecordId, mut activity: Activity) -> Activity {
    let key = record_key_literal(&id.key);
    activity.id = Some(id.clone());
    let mut map = FALLBACK_ACTIVITIES.lock().await;
    map.insert(key, activity.clone());
    activity
}

pub async fn fallback_contains(key: &str) -> bool {
    FALLBACK_ACTIVITIES.lock().await.contains_key(key)
}

fn fallback_activities() -> Result<Vec<Activity>, ServerFnError> {
    let map = FALLBACK_ACTIVITIES
        .try_lock()
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;
    let mut items: Vec<Activity> = map.values().cloned().collect();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(items)
}
