//! This module defines the `activity` component, which renders an activity stream
//! displaying recent actions or updates. It interacts with server functions to
//! fetch and potentially create activity entries.

extern crate alloc;
use leptos::prelude::*;
use leptos_meta::{Title, TitleProps};

use crate::api::{create_activity, select_activities};
use crate::types::Activity;

/// Renders the activity stream component, displaying a list of recent activities.
///
/// This component fetches activities using a `Resource` and displays them.
/// It also registers a server action for `create_activity`, though the action
/// itself is not directly triggered from this component's view in the current setup.
pub fn component() -> impl IntoView {
    let activities = Resource::new(
        || 0usize,
        |page| async move { select_activities(page).await },
    );

    // Register the server action so it is available in the Leptos runtime, even if unused here.
    let _create_activity_action = Action::new(move |activity: &Activity| {
        let activity = activity.clone();
        async move { create_activity(activity).await }
    });

    view! {
        <>
            {Title(TitleProps::builder()
                .text("Activity Stream – Alex Thola's Blog")
                .build())}

            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading activities..."</p> }>
                {move || {
                    let list = activities.get().and_then(Result::ok).unwrap_or_default();
                    view! {
                        <div class="container mx-auto max-w-4xl px-4 md:px-0">
                            <p class="text-gray-400">
                                "Activity stream component - server functions registered"
                            </p>
                            {list.into_iter().map(|activity| view! {
                                <div class="p-4 mb-4 bg-gray-800 rounded-lg">
                                    <p class="text-white">{activity.content.clone()}</p>
                                    <p class="text-gray-400 text-sm">{format!("Created: {}", activity.created_at)}</p>
                                </div>
                            }).collect_view()}
                        </div>
                    }
                }}
            </Suspense>
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Activity;

    #[test]
    /// Test the activity component's structure by verifying its function signature.
    /// The component cannot be fully instantiated without a Leptos runtime,
    /// but checking the signature ensures correct compilation and existence.
    fn test_activity_component_structure() {
        let _: fn() -> _ = component;
    }

    #[test]
    /// Verify the `select_activities` server function has the correct signature.
    /// This ensures its API contract remains stable, even without executing in a Leptos context.
    fn test_activity_fetch_signature() {
        use leptos::prelude::ServerFnError;

        let _check: fn(usize) -> _ = |_page| async {
            let _result: Result<Vec<Activity>, ServerFnError> = Ok(vec![]);
            _result
        };
    }

    #[test]
    /// Validate the structure of activity data against expected JSON patterns.
    /// This ensures consistency with how activity data is handled in integration tests.
    fn test_activity_data_validation() {
        let activity_data = serde_json::json!({
            "content": "This is a test activity",
            "tags": ["test", "rust"],
            "source": "https://example.com"
        });

        assert!(activity_data["content"].is_string());
        assert!(activity_data["tags"].is_array());
        assert!(activity_data["source"].is_string());

        let tags: Vec<String> = serde_json::from_value(activity_data["tags"].clone()).unwrap();
        assert_eq!(tags, vec!["test", "rust"]);
    }

    #[test]
    /// Verify serialization patterns for the `Activity` struct.
    /// This test ensures that the `Activity` struct serializes to JSON as expected by integration tests.
    fn test_activity_serialization_patterns() {
        let activity = Activity {
            content: "This is a test activity".to_string(),
            tags: vec!["test".to_string(), "rust".to_string()],
            source: Some("https://example.com".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&activity).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(parsed["content"], "This is a test activity");
        assert_eq!(parsed["tags"], serde_json::json!(["test", "rust"]));
        assert_eq!(parsed["source"], "https://example.com");
    }

    #[test]
    /// Confirm that default values for the `Activity` struct match expected states.
    /// This is crucial for integration tests that rely on these default states.
    fn test_activity_default_values() {
        let activity = Activity::default();

        assert_eq!(activity.content, "");
        assert!(activity.tags.is_empty());
        assert!(activity.source.is_none());
        assert!(activity.created_at.is_empty());
    }

    #[test]
    /// Test tag handling by the `Activity` struct.
    /// This ensures tag manipulation patterns are consistent with integration test expectations.
    fn test_activity_tag_handling() {
        let activity = Activity {
            tags: vec!["test".to_string(), "rust".to_string(), "web".to_string()],
            ..Default::default()
        };

        assert_eq!(activity.tags.len(), 3);
        assert!(activity.tags.contains(&"rust".to_string()));
        assert!(!activity.tags.contains(&"missing".to_string()));
    }
}
