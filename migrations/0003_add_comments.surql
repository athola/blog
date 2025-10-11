-- Add comments feature to blog engine
-- Migration: Add comment table and related functionality

-- Comments table: Stores user comments on posts
DEFINE TABLE OVERWRITE comment SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create WHERE $auth.id != NONE
        FOR update WHERE $auth.id = author_id
        FOR delete WHERE $auth.id = author_id OR $scope = "admin";

DEFINE FIELD OVERWRITE content ON comment TYPE string ASSERT $value != NONE AND string::len($value) <= 2000;
DEFINE FIELD OVERWRITE author_name ON comment TYPE string ASSERT $value != NONE AND string::len($value) <= 100;
DEFINE FIELD OVERWRITE author_email ON comment TYPE string ASSERT string::is_email($value);
DEFINE FIELD OVERWRITE author_id ON comment TYPE option<record>;
DEFINE FIELD OVERWRITE post_id ON comment TYPE record<post> ASSERT $value != NONE;
DEFINE FIELD OVERWRITE parent_comment ON comment TYPE option<record<comment>>;
DEFINE FIELD OVERWRITE is_approved ON comment TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE is_spam ON comment TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE created_at ON comment TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON comment TYPE datetime VALUE time::now();

-- Add comment count to post table
DEFINE FIELD OVERWRITE comment_count ON post TYPE int DEFAULT 0;

-- Event to update comment count when comments are added/removed
DEFINE EVENT OVERWRITE update_post_comment_count ON TABLE comment
WHEN $event = "CREATE" OR $event = "DELETE"
THEN {
    LET $post = $after.post_id OR $before.post_id;
    LET $count = (SELECT count() FROM comment WHERE post_id = $post AND is_approved = true GROUP ALL)[0].count OR 0;
    UPDATE post SET comment_count = $count WHERE id = $post;
};

-- Indexes for comment queries
DEFINE INDEX OVERWRITE idx_comment_post_id ON comment COLUMNS post_id;
DEFINE INDEX OVERWRITE idx_comment_approved ON comment COLUMNS is_approved;
DEFINE INDEX OVERWRITE idx_comment_created_at ON comment COLUMNS created_at;
DEFINE INDEX OVERWRITE idx_comment_parent ON comment COLUMNS parent_comment;