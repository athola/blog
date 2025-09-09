-- Add performance indexes for blog engine
-- Migration: Add indexes for common query patterns

-- Indexes for post table
DEFINE INDEX OVERWRITE idx_post_slug ON post COLUMNS slug UNIQUE;
DEFINE INDEX OVERWRITE idx_post_published ON post COLUMNS is_published;
DEFINE INDEX OVERWRITE idx_post_created_at ON post COLUMNS created_at;
DEFINE INDEX OVERWRITE idx_post_author ON post COLUMNS author;
DEFINE INDEX OVERWRITE idx_post_tags ON post COLUMNS tags;

-- Indexes for reference table  
DEFINE INDEX OVERWRITE idx_reference_published ON reference COLUMNS is_published;
DEFINE INDEX OVERWRITE idx_reference_tags ON reference COLUMNS tags;
DEFINE INDEX OVERWRITE idx_reference_tech_stack ON reference COLUMNS tech_stack;

-- Indexes for author table
DEFINE INDEX OVERWRITE idx_author_email ON author COLUMNS email UNIQUE;