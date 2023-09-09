-- Add up migration script here
ALTER TABLE cpo_cache
ADD COLUMN IF NOT EXISTS ccs_plug_count integer NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS type2_plug_count integer NOT NULL DEFAULT 0;
