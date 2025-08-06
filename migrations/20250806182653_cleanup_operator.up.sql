-- Add up migration script here
ALTER TABLE operator
DROP COLUMN supported_types,
DROP COLUMN ccs_plug_count,
DROP COLUMN type2_plug_count,
DROP COLUMN power_ac,
DROP COLUMN power_dc;
