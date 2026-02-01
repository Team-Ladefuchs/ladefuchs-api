CREATE TYPE link_banner_status_type AS ENUM('active', 'inactive');

ALTER TABLE link_banner
  ADD COLUMN status link_banner_status_type NOT NULL DEFAULT 'active';
