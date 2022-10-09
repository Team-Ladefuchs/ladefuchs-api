
ALTER TABLE if exists msp add column is_enabled boolean NOT NULL DEFAULT TRUE;
ALTER TABLE if exists msp add column msp_id uuid UNIQUE NOT NULL DEFAULT gen_random_uuid();

ALTER TYPE PlatformType RENAME TO plattformtype;

ALTER TABLE if exists affiliate_statistic RENAME COLUMN platform TO plattform;

ALTER TABLE if exists affiliate_statistic RENAME TO affiliate_state;

ALTER table if exists filter drop column comment;


ALTER table if exists tariff_image drop  is_ac_hoc bool;
