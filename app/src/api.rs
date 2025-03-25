use std::collections::BTreeMap;

use leptos::prelude::{server, ServerFnError};
use serde::{Deserialize, Serialize};

use crate::types::{Post, Reference};

#[server(endpoint = "/posts")]
pub async fn select_posts(#[server(default)] tags: Vec<String>) -> Result<Vec<Post>, ServerFnError> {
    use crate::types::AppState;
    use chrono::{DateTime, Utc};
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let mut query = String::from("SELECT *, author.* from post WHERE is_published = true ORDER BY created_at DESC;");
    if !tags.is_empty() {
        let tags = tags.iter().map(|tag| format!(r#""{}""#, tag)).collect::<Vec<_>>();
        query = format!(
            "SELECT *, author.* from post WHERE tags CONTAINSANY [{0}] ORDER BY created_at DESC;",
            tags.join(", ")
        );
    }

    let query = db.query(&query).await;

    if let Err(query_err) = query {
        return Err(ServerFnError::from(query_err));
    }

    let mut posts = query?.take::<Vec<Post>>(0)?;
    posts.iter_mut().for_each(|post| {
        let parsed_date = match DateTime::parse_from_rfc3339(&post.created_at) {
            Ok(date) => date,
            Err(date_err) => {
                return Err(ServerFnError::from(date_err));
            }
        };
        let date_time = parsed_date.with_timezone(&Utc);
        let naive_date = date_time.date_naive();
        let formatted_date = naive_date.format("%b %-d, %Y").to_string();
        post.created_at = formatted_date;
    });

    Ok(posts)
}

#[server(endpoint = "/tags")]
pub async fn select_tags() -> Result<BTreeMap<String, usize>, ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();

    let query = "
    LET $tags = SELECT tags FROM post;
    array::flatten($tags.map(|$t| $t.tags));
    ".to_string();
    let query = db.query(&query).await;

    if let Err(e) = query {
        return Err(ServerFnError::from(e));
    }

    let tags = query?.take::<Vec<String>>(1)?;
    let mut tag_map = BTreeMap::<String, usize>::new();
    for tag in tags {
        *tag_map.entry(tag).or_insert(0) += 1;
    }

    Ok(tag_map)
}

#[server(endpoint = "/post")]
pub async fn select_post(slug: String) -> Result<Post, ServerFnError> {
    use crate::types::AppState;
    use chrono::{DateTime, Utc};
    use leptos::prelude::expect_context;
    use markdown::process_markdown;

    let AppState { db, .. } = expect_context::<AppState>();

    let query = format!(r#"SELECT *, author.* from post WHERE slug = "{slug}""#);
    let query = db.query(&query).await;

    if let Err(query_err) = query {
        return Err(ServerFnError::from(query_err));
    }

    let post = query?.take::<Vec<Post>>(0)?;
    let mut post = match post.first() {
        Ok(first_post) => first_post.clone(),
        Err(post_err) => {
            return Err(ServerFnError::from(post_err));
        }
    };

    let date_time = DateTime::parse_from_rfc3339(&post.created_at)?.with_timezone(&Utc);
    let naive_date = date_time.date_naive();
    let formatted_date = naive_date.format("%b %-d").to_string();
    post.created_at = formatted_date;
    post.body = process_markdown(post.body.to_string()).await?;

    Ok(post)
}

#[server(endpoint = "/increment_views")]
pub async fn increment_views(id: String) -> Result<(), ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();

    let query = format!("UPDATE post:{0} SET total_views = total_views + 1;", id);
    let query = db.query(&query).await;

    if let Err(query_err) = query {
        return Err(ServerFnError::from(query_err));
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactRequest {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

#[server(endpoint = "/contact")]
pub async fn contact(data: ContactRequest) -> Result<(), ServerFnError> {
    use lettre::{
        message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
        Message, Tokio1Executor,
    };
    use std::env;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&env::var("SMTP_HOST")?)?
        .credentials(Credentials::new(env::var("SMTP_USER")?, env::var("SMTP_PASSWORD")?))
        .build::<Tokio1Executor>();

    let email = Message::builder()
        .from(env::var("SMTP_USER")?.parse()?)
        .to(env::var("SMTP_USER")?.parse()?)
        .subject(format!("{} - {}", data.email, data.subject))
        .header(ContentType::TEXT_HTML)
        .body(data.message)?;

    match mailer.send(email).await {
        Ok(_) => {
            tracing::info!("Email sent successfully");
            Ok(())
        }
        Err(email_err) => {
            tracing::error!("Failed to send email: {:?}", email_err);
            Err(ServerFnError::from(email_err))
        }
    }
}

#[server(endpoint = "/references")]
pub async fn select_references() -> Result<Vec<Reference>, ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();

    let query = "SELECT * from reference WHERE is_published = true ORDER BY created_at DESC;";
    let query = db.query(query).await;
    if let Err(query_err) = query {
        return Err(ServerFnError::from(query_err));
    }

    let references = query?.take::<Vec<Reference>>(0)?;
    Ok(references)
}
