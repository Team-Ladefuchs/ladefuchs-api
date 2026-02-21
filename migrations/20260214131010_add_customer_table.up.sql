CREATE TABLE IF NOT EXISTS customer (
    id SERIAL PRIMARY KEY,
    pub_id uuid UNIQUE NOT NULL DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE CHECK (name <> ''),
    total_impressions INT NOT NULL DEFAULT 0,
    created TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO customer (name, total_impressions) VALUES ('Ladefuchs', 0);
INSERT INTO customer (name, total_impressions) VALUES ('gruma', 0);
INSERT INTO customer (name, total_impressions) VALUES ('Lage der Nation', 0);
INSERT INTO customer (name, total_impressions) VALUES ('Strombock', 0);
INSERT INTO customer (name, total_impressions) VALUES ('BMW', 0);
INSERT INTO customer (name, total_impressions) VALUES ('Aral', 0);
INSERT INTO customer (name, total_impressions) VALUES ('Naturstrom', 0);

ALTER TABLE link_banner
    ADD COLUMN customer_id INT REFERENCES customer(id) ON DELETE RESTRICT ON UPDATE CASCADE;

CREATE INDEX idx_link_banner_customer_id ON link_banner(customer_id);

UPDATE link_banner SET customer_id = (SELECT id FROM customer WHERE name = 'Ladefuchs')
WHERE name IN ('ladefuchs_thg', 'mastodon', 'parkfuchs_werbung', 'shop_crazyelon', 'shop_denkzettel', 'shop_hoodie', 'shop_ihrewerbung');

UPDATE link_banner SET customer_id = (SELECT id FROM customer WHERE name = 'gruma')
WHERE name IN ('gruma9', 'gruma12');

UPDATE link_banner SET customer_id = (SELECT id FROM customer WHERE name = 'Strombock')
WHERE name = 'strombock';

UPDATE link_banner SET customer_id = (SELECT id FROM customer WHERE name = 'BMW')
WHERE name IN ('bmw-ionity', 'bmw-pulseshellmereon');

UPDATE link_banner SET customer_id = (SELECT id FROM customer WHERE name = 'Aral')
WHERE name = 'aral_pulse_2024-10';

UPDATE link_banner SET customer_id = (SELECT id FROM customer WHERE name = 'Lage der Nation')
WHERE name IN ('kennzeichene', 'kze_geldsparen');

UPDATE link_banner SET customer_id = (SELECT id FROM customer WHERE name = 'Naturstrom')
WHERE name = 'naturstrom_werbung';

UPDATE customer SET total_impressions = COALESCE((
    SELECT SUM(lb.impression) FROM link_banner lb WHERE lb.customer_id = customer.id
), 0);

ALTER TABLE link_banner
    ALTER COLUMN customer_id SET NOT NULL,
    DROP COLUMN impression;
