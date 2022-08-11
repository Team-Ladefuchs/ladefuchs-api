ALTER TABLE if exists charge_price RENAME COLUMN tarif_id TO tariff_id;

ALTER TABLE cpo add column updated timestamptz NOT NULL DEFAULT now();
