-- Add down migration script here
-- Drop the index
DROP INDEX IF EXISTS operator_cache_index;

-- Revert the data update
UPDATE operator
SET search = NULL;

-- Drop the trigger and function
DROP TRIGGER IF EXISTS operator_search_tsvector_update ON operator;
DROP FUNCTION IF EXISTS operator__search_tsvector();

-- Revert the table changes
ALTER TABLE IF EXISTS operator DROP COLUMN IF EXISTS search;
ALTER TABLE IF EXISTS operator RENAME TO cpo;

-- Revert the table rename
ALTER TABLE IF EXISTS cpo RENAME TO operator;

-- Drop the temporary table and related objects
DROP TABLE IF EXISTS cpo_cache CASCADE;
DROP TRIGGER IF EXISTS cpo_cache_tsvector_update ON cpo_cache;
DROP FUNCTION IF EXISTS cpo_cache_tsvector;
