-- add image_path column back to link_banner table
ALTER TABLE link_banner ADD COLUMN IF NOT EXISTS image_path TEXT NOT NULL DEFAULT '';
ALTER TABLE link_banner DROP CONSTRAINT IF EXISTS banner_image_fk;
ALTER TABLE link_banner DROP COLUMN if exists image;
ALTER TABLE link_banner DROP name;
