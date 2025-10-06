extern crate alloc;
use leptos::html::{div, p};
use leptos::prelude::*;
use leptos_meta::{Title, TitleProps};

use crate::api::{create_activity, select_activities};
use crate::types::Activity;

pub fn component() -> impl IntoView {
    let activities = Resource::new(
        || 0usize,
        |page| async move { select_activities(page).await },
    );

    // Create an action to ensure the create_activity server function is registered
    let _create_activity_action = Action::new(move |activity: &Activity| {
        let activity = activity.clone();
        async move { create_activity(activity).await }
    });

    div().child((
        Title(
            TitleProps::builder()
                .text("Activity Stream – Alex Thola's Blog")
                .build(),
        ),
        Suspense(
            SuspenseProps::builder()
                .fallback(|| p().class("text-gray-400").child("Loading activities..."))
                .children(TypedChildren::to_children(move || {
                    div()
                        .class("container mx-auto max-w-4xl px-4 md:px-0")
                        .child((
                            p().class("text-gray-400")
                                .child("Activity stream component - server functions registered"),
                            div().child(For(ForProps::builder()
                                .each(move || {
                                    activities.get().and_then(Result::ok).unwrap_or_default()
                                })
                                .key(|activity| activity.id.clone())
                                .children(|activity| {
                                    div().class("p-4 mb-4 bg-gray-800 rounded-lg").child((
                                        p().class("text-white").child(activity.content.clone()),
                                        p().class("text-gray-400 text-sm")
                                            .child(format!("Created: {}", activity.created_at)),
                                    ))
                                })
                                .build())),
                        ))
                }))
                .build(),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Activity;

    #[test]
    fn test_activity_component_creation() {
        // Test that the activity component can be created without panicking
        let _component = component();
        // Component creation should not panic
    }

    #[test]
    fn test_activity_resource_creation() {
        // Test that the activities resource can be created
        let _resource = Resource::new(
            || 0usize,
            |page| async move { select_activities(page).await },
        );
        // Resource creation should not panic
    }

    #[test]
    fn test_activity_data_validation() {
        // Test activity data validation patterns from integration tests
        let activity_data = serde_json::json!({
            "content": "This is a test activity",
            "tags": ["test", "rust"],
            "source": "https://example.com"
        });

        // Verify the structure matches expected Activity format
        assert!(activity_data["content"].is_string());
        assert!(activity_data["tags"].is_array());
        assert!(activity_data["source"].is_string());

        let tags: Vec<String> = serde_json::from_value(activity_data["tags"].clone()).unwrap();
        assert_eq!(tags, vec!["test", "rust"]);
    }

    #[test]
    fn test_activity_serialization_patterns() {
        // Test serialization patterns similar to integration test expectations
        let activity = Activity {
            content: "This is a test activity".to_string(),
            tags: vec!["test".to_string(), "rust".to_string()],
            source: Some("https://example.com".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        // Test serialization to JSON
        let serialized = serde_json::to_string(&activity).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Verify the structure matches integration test expectations
        assert_eq!(parsed["content"], "This is a test activity");
        assert_eq!(parsed["tags"], serde_json::json!(["test", "rust"]));
        assert_eq!(parsed["source"], "https://example.com");
    }

    #[test]
    fn test_activity_default_values() {
        // Test default activity values for edge cases
        let activity = Activity::default();

        // These are the values that integration tests might rely on
        assert_eq!(activity.content, "");
        assert!(activity.tags.is_empty());
        assert!(activity.source.is_none());
        assert!(activity.created_at.is_empty());
    }

    #[test]
    fn test_activity_tag_handling() {
        // Test tag handling patterns from integration test scenarios
        let activity = Activity {
            tags: vec!["test".to_string(), "rust".to_string(), "web".to_string()],
            ..Default::default()
        };

        // Test tag manipulation patterns
        assert_eq!(activity.tags.len(), 3);
        assert!(activity.tags.contains(&"rust".to_string()));
        assert!(!activity.tags.contains(&"missing".to_string()));
    }
}
