DEFINE EVENT OVERWRITE post_activity ON TABLE post
WHEN ($event = "UPDATE" || $event = "CREATE")
    && $after.is_published = true
    && ($before.is_published = false || $before.is_published = NONE)
THEN {
    LET $slug_source = IF $after.slug = NONE || $after.slug = "" THEN string::slug($after.title) ELSE $after.slug END;
    LET $slug_without_mid_post = string::replace($slug_source, "-post-", "-");
    LET $slug_cleaned = IF string::ends_with($slug_without_mid_post, "-post") THEN string::slice($slug_without_mid_post, 0, string::len($slug_without_mid_post) - 5) ELSE $slug_without_mid_post END;
    LET $post_url = "https://alexthola.com/post/" + $slug_source;
    LET $clean_summary = string::trim($after.summary);
    LET $summary_limited = string::slice($clean_summary, 0, 180);
    LET $summary = IF string::len($clean_summary) > 180 THEN string::trim($summary_limited) + "..." ELSE $clean_summary END;
    LET $summary_segment = IF $summary = "" THEN "" ELSE " - " + $summary END;
    LET $activity_content = "Published on Alex Thola's blog: " + $after.title + $summary_segment + " (" + $post_url + ")";
    LET $activity_key = "post-" + $slug_cleaned;

    CREATE activity CONTENT {
        content: $activity_content,
        tags: ["new post", "blog"],
        source: $post_url,
        created_at: time::now()
    };
};
