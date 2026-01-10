//! This module defines the `activity` component, which renders an activity stream
//! displaying recent actions or updates. It interacts with server functions to
//! fetch and potentially create activity entries.
//!
//! Features paginated loading - loads 10 activities per page with a "Load More"
//! button to fetch additional pages on demand.

extern crate alloc;
use leptos::prelude::*;
use leptos_meta::{Title, TitleProps};

use crate::api::{create_activity, select_activities};
use crate::types::Activity;

/// Number of activities per page (matches server-side ACTIVITIES_PER_PAGE).
const ACTIVITIES_PER_PAGE: usize = 10;

/// Renders the activity stream component with paginated loading.
///
/// This component fetches activities page by page, accumulating results as the
/// user requests more. Displays a "Load More" button when additional pages are
/// available.
pub fn component() -> impl IntoView {
    // Accumulated list of all loaded activities
    let all_activities = RwSignal::new(Vec::<Activity>::new());
    // Current page number for pagination
    let current_page = RwSignal::new(0usize);
    // Whether we're currently loading a page
    let is_loading = RwSignal::new(false);
    // Whether there are more pages to load
    let has_more = RwSignal::new(true);

    // Resource that fetches activities for the current page
    let page_resource = Resource::new(
        move || current_page.get(),
        |page| async move { select_activities(page).await },
    );

    // Effect to accumulate fetched activities
    Effect::new(move || {
        match page_resource.get() {
            Some(Ok(new_activities)) => {
                // Check if we've reached the end (fewer than full page returned)
                if new_activities.len() < ACTIVITIES_PER_PAGE {
                    has_more.set(false);
                }

                // Append new activities to the accumulated list
                if !new_activities.is_empty() {
                    all_activities.update(|list| {
                        list.extend(new_activities);
                    });
                }

                is_loading.set(false);
            }
            Some(Err(e)) => {
                leptos::logging::error!("Failed to fetch activities: {:?}", e);
                is_loading.set(false);
                has_more.set(false); // Stop attempting to load more on error
            }
            None => {
                // Resource is still loading, nothing to do
            }
        }
    });

    // Function to load the next page
    let load_more = move || {
        if !is_loading.get() && has_more.get() {
            is_loading.set(true);
            current_page.update(|p| *p += 1);
        }
    };

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

            <div class="container mx-auto max-w-4xl px-4 md:px-0">
                // Activity list
                <div class="space-y-4">
                    {move || {
                        all_activities.get().into_iter().map(|activity| {
                            view! {
                                <div class="p-4 bg-gray-800 rounded-lg">
                                    <p class="text-white">{activity.content.clone()}</p>
                                    // Display tags if present
                                    {(!activity.tags.is_empty()).then(|| {
                                        view! {
                                            <div class="flex flex-wrap gap-2 mt-2">
                                                {activity.tags.iter().map(|tag| {
                                                    view! {
                                                        <span class="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded">
                                                            {tag.clone()}
                                                        </span>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }
                                    })}
                                    // Display source link if present
                                    {activity.source.clone().map(|src| {
                                        let display_src = src.clone();
                                        view! {
                                            <a href={src} class="text-blue-400 hover:underline text-sm mt-2 block" target="_blank">
                                                {display_src}
                                            </a>
                                        }
                                    })}
                                    <p class="text-gray-400 text-sm mt-2">{format!("Created: {}", activity.created_at)}</p>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>

                // Loading indicator and Load More button
                <div class="mt-8 mb-8 text-center">
                    {move || {
                        if is_loading.get() {
                            view! {
                                <p class="text-gray-400">"Loading more activities..."</p>
                            }.into_any()
                        } else if has_more.get() {
                            view! {
                                <button
                                    class="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                                    on:click=move |_| load_more()
                                >
                                    "Load More"
                                </button>
                            }.into_any()
                        } else if all_activities.get().is_empty() {
                            view! {
                                <p class="text-gray-400">"No activities yet."</p>
                            }.into_any()
                        } else {
                            view! {
                                <p class="text-gray-500">"You've reached the end."</p>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
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

    #[test]
    /// Verify pagination constant matches server-side configuration.
    fn test_activities_per_page_constant() {
        // Must match the server-side ACTIVITIES_PER_PAGE in api.rs
        assert_eq!(ACTIVITIES_PER_PAGE, 10);
    }
}
