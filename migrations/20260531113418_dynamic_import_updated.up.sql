ALTER TABLE charging_location
    ADD COLUMN updated TIMESTAMPTZ NOT NULL DEFAULT now();
CREATE INDEX idx_charging_location_updated ON charging_location (updated);

ALTER TABLE location_dynamic_price
    ADD COLUMN updated TIMESTAMPTZ NOT NULL DEFAULT now();
CREATE INDEX idx_location_dynamic_price_updated ON location_dynamic_price (updated);

CREATE INDEX idx_dynamic_charge_price_updated ON dynamic_charge_price (updated);
