-- Initial schema migration for blog engine
-- Creates: author, post, reference, and script_migration tables

-- Author table: Stores author information
DEFINE TABLE OVERWRITE author TYPE NORMAL SCHEMAFULL
    PERMISSIONS 
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE name ON author TYPE string ASSERT $value != NONE;
DEFINE FIELD OVERWRITE bio ON author TYPE option<string>;
DEFINE FIELD OVERWRITE email ON author TYPE string ASSERT string::is_email($value);
DEFINE FIELD OVERWRITE linkedin ON author TYPE option<string>;
DEFINE FIELD OVERWRITE twitter ON author TYPE option<string>;
DEFINE FIELD OVERWRITE github ON author TYPE option<string>;

-- Post table: Stores blog posts
DEFINE TABLE OVERWRITE post TYPE NORMAL SCHEMAFULL 
    PERMISSIONS 
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE title ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD OVERWRITE summary ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD OVERWRITE body ON post TYPE string ASSERT $value != NONE;
DEFINE FIELD OVERWRITE slug ON post TYPE option<string>;
DEFINE FIELD OVERWRITE tags ON post TYPE array<string> ASSERT $value != NONE;
DEFINE FIELD OVERWRITE author ON post TYPE record<author> ASSERT $value != NONE;
DEFINE FIELD OVERWRITE read_time ON post TYPE option<int>;
DEFINE FIELD OVERWRITE total_views ON post TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE created_at ON post TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON post TYPE datetime VALUE time::now();
DEFINE FIELD OVERWRITE is_published ON post TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE header_image ON post TYPE option<string>;
DEFINE FIELD OVERWRITE show_cta ON post TYPE bool DEFAULT false;

-- Reference table: Stores reference materials/links
DEFINE TABLE OVERWRITE reference SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE title ON reference TYPE string;
DEFINE FIELD OVERWRITE description ON reference TYPE string;
DEFINE FIELD OVERWRITE url ON reference TYPE string;
DEFINE FIELD OVERWRITE tags ON reference TYPE array<string>;
DEFINE FIELD OVERWRITE tech_stack ON reference TYPE array<string>;
DEFINE FIELD OVERWRITE teck_stack_percentage ON reference TYPE array<int>;
DEFINE FIELD OVERWRITE is_published ON reference TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE created_at ON reference TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON reference TYPE datetime VALUE time::now();

-- Script migration tracking table
DEFINE TABLE OVERWRITE script_migration SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete NONE;

DEFINE FIELD OVERWRITE script_name ON script_migration TYPE string;
DEFINE FIELD OVERWRITE executed_at ON script_migration TYPE datetime VALUE time::now() READONLY;

-- Events for automatic field updates
DEFINE EVENT OVERWRITE post_read_time ON TABLE post 
WHEN ($event = "UPDATE" || $event = "CREATE") && $before.body != $after.body 
THEN {
    LET $read_time = array::len(string::words($after.body)) / 200;
    UPDATE post SET read_time = math::max([1, $read_time]) WHERE id = $after.id;
};

DEFINE EVENT OVERWRITE post_slug ON TABLE post 
WHEN ($event = "CREATE" OR $event = "UPDATE") && $before.title != $after.title 
THEN (
    UPDATE post SET slug = string::slug($after.title) WHERE id = $after.id
);