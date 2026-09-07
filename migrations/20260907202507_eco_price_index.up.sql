CREATE INDEX IF NOT EXISTS idx_price_tariff_id
	ON eco_movement.price (tariff_id, id);

CREATE INDEX IF NOT EXISTS idx_connector_price_pricing_location
	ON eco_movement.connector_price (pricing_id, location_id)
	INCLUDE (connector_id, evse_uid);

ANALYZE eco_movement.price;
ANALYZE eco_movement.connector_price;
