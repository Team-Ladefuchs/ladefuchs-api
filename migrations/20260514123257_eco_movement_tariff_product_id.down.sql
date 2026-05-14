ALTER TABLE eco_movement.tariff ADD CONSTRAINT tariff_name_provider_name_key UNIQUE (name, provider_name);
ALTER TABLE eco_movement.tariff DROP COLUMN product_id;
