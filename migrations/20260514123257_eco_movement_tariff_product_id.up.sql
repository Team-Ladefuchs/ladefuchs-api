ALTER TABLE eco_movement.tariff ADD COLUMN product_id UUID UNIQUE;
ALTER TABLE eco_movement.tariff DROP CONSTRAINT tariff_name_provider_name_key;
