use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;
#[cfg(feature = "ssr")]
use surrealdb::{Surreal, engine::remote::http::Client};

#[cfg(feature = "ssr")]
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub db: std::sync::Arc<Surreal<Client>>,
    pub leptos_options: std::sync::Arc<LeptosOptions>,
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.as_ref().clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    pub id: Thing,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub github: Option<String>,
}

impl Default for Author {
    fn default() -> Self {
        Self {
            id: Thing::from(("author", "0")),
            name: String::new(),
            email: String::new(),
            bio: None,
            linkedin: None,
            twitter: None,
            github: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Post {
    pub id: Thing,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub tags: Vec<String>,
    pub author: Author,
    pub read_time: usize,
    pub total_views: usize,
    pub slug: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_published: bool,
    pub header_image: Option<String>,
}

impl Default for Post {
    fn default() -> Self {
        Self {
            id: Thing::from(("post", "0")),
            title: String::new(),
            summary: String::new(),
            body: String::new(),
            tags: vec![],
            author: Author::default(),
            read_time: 0,
            total_views: 0,
            slug: None,
            created_at: String::new(),
            updated_at: String::new(),
            is_published: true,
            header_image: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reference {
    pub id: Thing,
    pub title: String,
    pub description: String,
    pub url: String,
    pub tags: Vec<String>,
    pub tech_stack: Vec<String>,
    pub teck_stack_percentage: Vec<u8>,
    pub created_at: String,
    pub updated_at: String,
    pub is_published: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Activity {
    #[serde(default = "default_thing")]
    pub id: Thing,
    pub content: String,
    pub tags: Vec<String>,
    pub source: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            id: default_thing(),
            content: String::new(),
            tags: Vec::new(),
            source: None,
            created_at: String::new(),
        }
    }
}

fn default_thing() -> Thing {
    Thing::from(("activity", "0"))
}

#[cfg(test)]
mod activity_type_tests {
    use super::*;
    use serde_json::json;

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
}
