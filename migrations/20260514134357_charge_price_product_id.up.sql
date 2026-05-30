ALTER TABLE charge_price ADD COLUMN product_id UUID;
CREATE INDEX idx_charge_price_product_id ON charge_price (product_id);
