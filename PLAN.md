# Roadmap

This document outlines planned features and technical improvements for the blog engine.

## Q1 2026: Reader Experience

This quarter improves the core reading experience.

-   **Theme Toggle**: Implement a dark/light mode toggle to reduce eye strain for readers in different lighting conditions. This will use CSS custom properties and a simple JavaScript toggle stored in `localStorage`.
-   **Syntax Highlighting**: Add server-side syntax highlighting for code blocks to improve readability and performance. This will likely involve integrating a Rust library like `syntect` during the Markdown rendering process.
-   **Post Search**: Implement a fast, server-side search feature. This will allow readers to find posts by keyword without relying on a third-party search provider. The initial implementation will likely use a simple full-text search index within SurrealDB.
-   **Related Articles**: Display a list of related posts at the end of each article to encourage further reading. The relationship will be determined by shared tags or content similarity.

## Q2 2026: Community Features

This quarter adds features for reader interaction.

-   **Comments**: A self-hosted commenting system will be added to allow for discussions on posts. The implementation will focus on privacy and performance, avoiding third-party tracking. It will be built directly within the existing Rust and SurrealDB stack.
-   **Social Sharing**: Implement lightweight social sharing links that use direct URLs rather than third-party JavaScript widgets. This will allow readers to share content easily without sacrificing page load performance or privacy.
-   **Newsletter Signup**: Add a newsletter signup form, likely integrated with a self-hosted or privacy-focused email provider. The focus will be on a non-intrusive design.

## Backlog

These items are under consideration for future quarters.

-   **Admin Interface**: A simple, secure web interface for creating, editing, and managing posts. This would reduce the need to interact directly with the database or filesystem for content management.
-   **Offline Reading**: Implement basic Progressive Web App (PWA) functionality to allow articles to be read offline. This would improve the experience for readers with intermittent connectivity.
-   **AI-Powered Suggestions**: Experiment with using local, privacy-preserving AI models to suggest tags or generate summaries for new posts. This is a low-priority exploration, contingent on the maturity and performance of local AI models.