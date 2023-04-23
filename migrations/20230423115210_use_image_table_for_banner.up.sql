-- Add up migration script here
ALTER table link_banner 
	add column if not exists image int constraint banner_image_fk references image(id) on delete set null;
ALTER TABLE link_banner 
	DROP COLUMN IF EXISTS image_path;
ALTER TABLE link_banner 
	ADD COLUMN IF NOT EXISTS name text not null default '';
