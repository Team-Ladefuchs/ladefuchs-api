-- Add down migration script here
ALTER TABLE IF EXISTS cpo_cache
DROP COLUMN IF EXISTS ccs_plug_count,
DROP COLUMN IF EXISTS type2_plug_count;
