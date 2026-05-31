ALTER TABLE dynamic_charge_price ADD COLUMN product_id UUID;
CREATE INDEX idx_dynamic_charge_price_product_id ON dynamic_charge_price (product_id);
