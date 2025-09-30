#[path = "activity_test_api.rs"]
mod activity_test_api;

use activity_test_api::{create_activity, select_activities, TestDb};
use app::types::Activity;
use serde_json::json;
use surrealdb::engine::local::Mem;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

#[cfg(test)]
mod activity_unit_tests {

    use super::*;

    // Mock database for testing
    async fn setup_mock_db() -> Surreal<TestDb> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    // === Activity Type Validation and Serialization Tests ===

    #[test]
    fn test_activity_default_values() {
        let activity = Activity::default();

        assert_eq!(activity.content, "");
        assert_eq!(activity.tags, Vec::<String>::new());
        assert_eq!(activity.source, None);
        assert_eq!(activity.created_at, "");
        assert_eq!(activity.id, Thing::from(("activity", "0")));
    }

    #[test]
    fn test_activity_partial_equality() {
        let activity1 = Activity {
            content: "Test content".to_string(),
            tags: vec!["test".to_string()],
            source: Some("https://example.com".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let activity2 = Activity {
            content: "Test content".to_string(),
            tags: vec!["test".to_string()],
            source: Some("https://example.com".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        assert_eq!(activity1, activity2);
    }

    #[test]
    fn test_activity_inequality() {
        let activity1 = Activity {
            content: "Content 1".to_string(),
            ..Default::default()
        };

        let activity2 = Activity {
            content: "Content 2".to_string(),
            ..Default::default()
        };

        assert_ne!(activity1, activity2);
    }

    #[test]
    fn test_activity_serialization_roundtrip() {
        let original_activity = Activity {
            content: "Test activity content".to_string(),
            tags: vec!["rust".to_string(), "testing".to_string()],
            source: Some("https://github.com".to_string()),
            created_at: "2023-12-25T15:30:00Z".to_string(),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&original_activity).expect("Serialization failed");
        let deserialized: Activity =
            serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(original_activity.content, deserialized.content);
        assert_eq!(original_activity.tags, deserialized.tags);
        assert_eq!(original_activity.source, deserialized.source);
        assert_eq!(original_activity.created_at, deserialized.created_at);
        assert_eq!(original_activity.id, deserialized.id);
    }

    #[test]
    fn test_activity_json_structure() {
        let activity = Activity {
            content: "JSON test".to_string(),
            tags: vec!["json".to_string()],
            source: Some("https://json.org".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let json_value = serde_json::to_value(activity).expect("JSON conversion failed");

        assert!(json_value.is_object());
        assert_eq!(json_value["content"], "JSON test");
        assert_eq!(json_value["tags"], json!(["json"]));
        assert_eq!(json_value["source"], "https://json.org");
        assert_eq!(json_value["created_at"], "2023-01-01T00:00:00Z");
        assert_eq!(json_value["id"]["tb"], "activity");
        assert_eq!(json_value["id"]["id"]["String"], "0");
    }

    #[test]
    fn test_activity_with_empty_tags() {
        let activity = Activity {
            content: "No tags".to_string(),
            tags: Vec::new(),
            source: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&activity).unwrap();
        let deserialized: Activity = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.tags.is_empty());
        assert_eq!(deserialized.content, "No tags");
    }

    #[test]
    fn test_activity_with_none_source() {
        let activity = Activity {
            content: "No source".to_string(),
            tags: vec!["test".to_string()],
            source: None,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&activity).unwrap();
        let deserialized: Activity = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.source.is_none());
        assert_eq!(deserialized.content, "No source");
    }

    #[test]
    fn test_activity_with_special_characters() {
        let activity = Activity {
            content: "Special chars: áéíóú ñ ¿¡ 🚀".to_string(),
            tags: vec!["español".to_string(), "unicode".to_string()],
            source: Some("https://example.com/path?query=value&other=123".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&activity).unwrap();
        let deserialized: Activity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.content, "Special chars: áéíóú ñ ¿¡ 🚀");
        assert_eq!(
            deserialized.tags,
            vec!["español".to_string(), "unicode".to_string()]
        );
        assert_eq!(
            deserialized.source,
            Some("https://example.com/path?query=value&other=123".to_string())
        );
    }

    #[test]
    fn test_activity_clone() {
        let original = Activity {
            content: "Clone test".to_string(),
            tags: vec!["clone".to_string()],
            source: Some("https://clone.com".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.content, cloned.content);
        assert_eq!(original.tags, cloned.tags);
        assert_eq!(original.source, cloned.source);
    }

    #[test]
    fn test_activity_debug_format() {
        let activity = Activity {
            content: "Debug test".to_string(),
            tags: vec!["debug".to_string()],
            source: Some("https://debug.com".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let debug_string = format!("{:?}", activity);

        assert!(debug_string.contains("Activity"));
        assert!(debug_string.contains("Debug test"));
        assert!(debug_string.contains("debug"));
        assert!(debug_string.contains("https://debug.com"));
    }

    // === Activity Creation Tests ===

    #[tokio::test]
    async fn test_create_activity_basic() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Thing::from(("activity", "test_id")),
            content: "This is a test activity".to_string(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        // Verify the activity was created in the mock database
        let created_activity: Option<Activity> = db.select(("activity", "test_id")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(created_activity.unwrap().content, activity.content);
    }

    #[tokio::test]
    async fn test_create_activity_with_tags() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Thing::from(("activity", "tagged_activity")),
            content: "Activity with tags".to_string(),
            tags: vec!["rust".to_string(), "testing".to_string(), "tdd".to_string()],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "tagged_activity")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(created.content, activity.content);
        assert_eq!(created.tags, activity.tags);
    }

    #[tokio::test]
    async fn test_create_activity_with_source() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Thing::from(("activity", "sourced_activity")),
            content: "Activity with source".to_string(),
            source: Some("https://github.com/rust-lang/rust".to_string()),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "sourced_activity")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(created.content, activity.content);
        assert_eq!(created.source, activity.source);
    }

    #[tokio::test]
    async fn test_create_activity_with_empty_content() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Thing::from(("activity", "empty_content")),
            content: "".to_string(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "empty_content")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(created_activity.unwrap().content, "");
    }

    #[tokio::test]
    async fn test_create_activity_with_long_content() {
        let db = setup_mock_db().await;
        let long_content = "a".repeat(10000); // 10KB of content
        let activity = Activity {
            id: Thing::from(("activity", "long_content")),
            content: long_content.clone(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "long_content")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(created_activity.unwrap().content, long_content);
    }

    #[tokio::test]
    async fn test_create_activity_with_special_characters() {
        let db = setup_mock_db().await;
        let special_content = "Special chars: áéíóú ñ ¿¡ 🚀 \n\t\r\"'\\";
        let activity = Activity {
            id: Thing::from(("activity", "special_chars")),
            content: special_content.to_string(),
            tags: vec!["español".to_string(), "unicode".to_string()],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "special_chars")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(created.content, special_content);
        assert_eq!(
            created.tags,
            vec!["español".to_string(), "unicode".to_string()]
        );
    }

    #[tokio::test]
    async fn test_create_activity_with_unicode_tags() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Thing::from(("activity", "unicode_tags")),
            content: "Unicode tags test".to_string(),
            tags: vec![
                "中文".to_string(),
                "日本語".to_string(),
                "한국어".to_string(),
            ],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "unicode_tags")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(
            created.tags,
            vec![
                "中文".to_string(),
                "日本語".to_string(),
                "한국어".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn test_create_activity_with_empty_tags() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Thing::from(("activity", "empty_tags")),
            content: "Empty tags test".to_string(),
            tags: Vec::new(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "empty_tags")).await.unwrap();
        assert!(created_activity.is_some());
        assert!(created_activity.unwrap().tags.is_empty());
    }

    #[tokio::test]
    async fn test_create_activity_with_invalid_url_source() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Thing::from(("activity", "invalid_url")),
            content: "Invalid URL test".to_string(),
            source: Some("not-a-valid-url".to_string()),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "invalid_url")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(
            created_activity.unwrap().source,
            Some("not-a-valid-url".to_string())
        );
    }

    #[tokio::test]
    async fn test_create_multiple_activities() {
        let db = setup_mock_db().await;
        let activities = vec![
            Activity {
                id: Thing::from(("activity", "multi_1")),
                content: "First activity".to_string(),
                created_at: "2023-01-01T12:00:00Z".to_string(),
                ..Default::default()
            },
            Activity {
                id: Thing::from(("activity", "multi_2")),
                content: "Second activity".to_string(),
                tags: vec!["test".to_string()],
                created_at: "2023-01-01T12:01:00Z".to_string(),
                ..Default::default()
            },
            Activity {
                id: Thing::from(("activity", "multi_3")),
                content: "Third activity".to_string(),
                source: Some("https://example.com".to_string()),
                created_at: "2023-01-01T12:02:00Z".to_string(),
                ..Default::default()
            },
        ];

        for activity in activities {
            let result = create_activity(&db, activity.clone()).await;
            assert!(result.is_ok());
        }

        // Verify all activities were created
        for i in 1..=3 {
            let created_activity: Option<Activity> = db
                .select(("activity", format!("multi_{}", i)))
                .await
                .unwrap();
            assert!(created_activity.is_some());
        }
    }

    // === Activity Selection Tests ===

    #[tokio::test]
    async fn test_select_activities_basic() {
        let db = setup_mock_db().await;
        // Create some test activities
        for i in 0..5 {
            let activity = Activity {
                id: Thing::from(("activity".to_owned(), format!("test_id_{}", i))),
                content: format!("Activity {}", i),
                created_at: format!("2023-01-01T12:00:0{}Z", i),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 5);
        assert_eq!(activities[0].content, "Activity 4");
    }

    #[tokio::test]
    async fn test_select_activities_with_pagination() {
        let db = setup_mock_db().await;
        // Create 25 test activities
        for i in 0..25 {
            let activity = Activity {
                id: Thing::from(("activity".to_owned(), format!("page_test_{}", i))),
                content: format!("Page test activity {}", i),
                created_at: format!("2023-01-01T12:{:02}:00Z", i),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        // Test first page (should have 10 activities)
        let page1 = select_activities(&db, 0).await.unwrap();
        assert_eq!(page1.len(), 10);
        assert_eq!(page1[0].content, "Page test activity 24"); // Most recent first

        // Test second page (should have 10 activities)
        let page2 = select_activities(&db, 1).await.unwrap();
        assert_eq!(page2.len(), 10);
        assert_eq!(page2[0].content, "Page test activity 14");

        // Test third page (should have 5 activities)
        let page3 = select_activities(&db, 2).await.unwrap();
        assert_eq!(page3.len(), 5);
        assert_eq!(page3[0].content, "Page test activity 4");

        // Test fourth page (should be empty)
        let page4 = select_activities(&db, 3).await.unwrap();
        assert_eq!(page4.len(), 0);
    }

    #[tokio::test]
    async fn test_select_activities_ordering() {
        let db = setup_mock_db().await;
        // Create activities with different timestamps
        let activities_data = vec![
            ("2023-01-01T10:00:00Z", "Oldest activity"),
            ("2023-01-01T11:00:00Z", "Middle activity"),
            ("2023-01-01T12:00:00Z", "Newest activity"),
        ];

        for (timestamp, content) in activities_data {
            let activity = Activity {
                id: Thing::from((
                    "activity".to_owned(),
                    content.replace(" ", "_").to_lowercase(),
                )),
                content: content.to_string(),
                created_at: timestamp.to_string(),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 3);

        // Should be ordered by created_at DESC (newest first)
        assert_eq!(activities[0].content, "Newest activity");
        assert_eq!(activities[1].content, "Middle activity");
        assert_eq!(activities[2].content, "Oldest activity");
    }

    #[tokio::test]
    async fn test_select_activities_with_same_timestamp() {
        let db = setup_mock_db().await;
        // Create activities with the same timestamp
        let same_timestamp = "2023-01-01T12:00:00Z";
        let activities_data = vec![
            ("same_time_1", "First same time"),
            ("same_time_2", "Second same time"),
            ("same_time_3", "Third same time"),
        ];

        for (id, content) in activities_data {
            let activity = Activity {
                id: Thing::from(("activity", id)),
                content: content.to_string(),
                created_at: same_timestamp.to_string(),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 3);

        // All should have the same timestamp
        for activity in &activities {
            assert_eq!(activity.created_at, same_timestamp);
        }
    }

    #[tokio::test]
    async fn test_select_activities_with_various_content() {
        let db = setup_mock_db().await;
        // Create activities with different content types
        let activities_data = vec![
            ("empty_content", "".to_string()),
            ("short_content", "Hi".to_string()),
            (
                "medium_content",
                "This is a medium length activity content".to_string(),
            ),
            ("long_content", "a".repeat(1000)),
            (
                "unicode_content",
                "Unicode: áéíóú ñ ¿¡ 🚀 中文 日本語 한국어".to_string(),
            ),
            (
                "special_chars",
                "Special: !@#$%^&*()_+-=[]{};':\",./<>?`~".to_string(),
            ),
            (
                "whitespace",
                "  Multiple   spaces   and\ttabs\nnewlines  ".to_string(),
            ),
        ];

        for (id, content) in activities_data {
            let activity = Activity {
                id: Thing::from(("activity", id)),
                content,
                created_at: "2023-01-01T12:00:00Z".to_string(),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 7);

        // Verify all content types are preserved
        let contents: Vec<String> = activities.iter().map(|a| a.content.clone()).collect();
        assert!(contents.contains(&"".to_string()));
        assert!(contents.contains(&"Hi".to_string()));
        assert!(contents.contains(&"This is a medium length activity content".to_string()));
        assert!(contents.contains(&"a".repeat(1000)));
        assert!(contents.contains(&"Unicode: áéíóú ñ ¿¡ 🚀 中文 日本語 한국어".to_string()));
        assert!(contents.contains(&"Special: !@#$%^&*()_+-=[]{};':\",./<>?`~".to_string()));
        assert!(contents.contains(&"  Multiple   spaces   and\ttabs\nnewlines  ".to_string()));
    }

    #[tokio::test]
    async fn test_select_activities_with_tags_and_sources() {
        let db = setup_mock_db().await;
        // Create activities with various tags and sources
        let activities_data = vec![
            Activity {
                id: Thing::from(("activity", "tagged_1")),
                content: "Activity with tags".to_string(),
                tags: vec!["rust".to_string(), "web".to_string()],
                source: None,
                created_at: "2023-01-01T12:00:00Z".to_string(),
            },
            Activity {
                id: Thing::from(("activity", "sourced_1")),
                content: "Activity with source".to_string(),
                tags: Vec::new(),
                source: Some("https://github.com".to_string()),
                created_at: "2023-01-01T12:01:00Z".to_string(),
            },
            Activity {
                id: Thing::from(("activity", "both_1")),
                content: "Activity with both".to_string(),
                tags: vec!["fullstack".to_string()],
                source: Some("https://example.com".to_string()),
                created_at: "2023-01-01T12:02:00Z".to_string(),
            },
            Activity {
                id: Thing::from(("activity", "neither_1")),
                content: "Activity with neither".to_string(),
                tags: Vec::new(),
                source: None,
                created_at: "2023-01-01T12:03:00Z".to_string(),
            },
        ];

        for activity in activities_data {
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 4);

        // Verify tags and sources are preserved
        for activity in &activities {
            match activity.id.id.to_string().as_str() {
                "tagged_1" => {
                    assert_eq!(activity.tags, vec!["rust".to_string(), "web".to_string()]);
                    assert!(activity.source.is_none());
                }
                "sourced_1" => {
                    assert!(activity.tags.is_empty());
                    assert_eq!(activity.source, Some("https://github.com".to_string()));
                }
                "both_1" => {
                    assert_eq!(activity.tags, vec!["fullstack".to_string()]);
                    assert_eq!(activity.source, Some("https://example.com".to_string()));
                }
                "neither_1" => {
                    assert!(activity.tags.is_empty());
                    assert!(activity.source.is_none());
                }
                _ => panic!("Unexpected activity ID"),
            }
        }
    }

    #[tokio::test]
    async fn test_select_activities_empty_database() {
        let db = setup_mock_db().await;
        let activities = select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 0);

        // Test multiple pages on empty database
        let activities_page2 = select_activities(&db, 1).await.unwrap();
        assert_eq!(activities_page2.len(), 0);

        let activities_page10 = select_activities(&db, 10).await.unwrap();
        assert_eq!(activities_page10.len(), 0);
    }

    #[tokio::test]
    async fn test_select_activities_large_page_number() {
        let db = setup_mock_db().await;
        // Create only 5 activities
        for i in 0..5 {
            let activity = Activity {
                id: Thing::from(("activity".to_owned(), format!("large_page_{}", i))),
                content: format!("Activity {}", i),
                created_at: format!("2023-01-01T12:00:0{}Z", i),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        // Test with a very large page number
        let activities = select_activities(&db, 1000).await.unwrap();
        assert_eq!(activities.len(), 0);
    }

    #[tokio::test]
    async fn test_select_activities_with_empty_database() {
        let db = setup_mock_db().await;
        let activities = select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 0);
    }
}
