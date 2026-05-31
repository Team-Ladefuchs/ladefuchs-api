DROP INDEX idx_dynamic_charge_price_updated;

DROP INDEX idx_location_dynamic_price_updated;
ALTER TABLE location_dynamic_price DROP COLUMN updated;

DROP INDEX idx_charging_location_updated;
ALTER TABLE charging_location DROP COLUMN updated;
