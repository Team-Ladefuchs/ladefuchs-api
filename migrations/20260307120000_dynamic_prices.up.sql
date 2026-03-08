CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TYPE day_of_week AS ENUM ('monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday');

CREATE TABLE charging_location (
    id BIGSERIAL PRIMARY KEY,
    eco_movement_id UUID UNIQUE NOT NULL,
    operator_id INT NOT NULL REFERENCES operator(id) ON DELETE CASCADE,
    geo GEOGRAPHY(Point, 4326) NOT NULL,
    address TEXT,
    city TEXT,
    postal_code TEXT
);
CREATE INDEX idx_charging_location_geo ON charging_location USING GIST (geo);
CREATE INDEX idx_charging_location_operator ON charging_location (operator_id);

CREATE TABLE dynamic_charge_price (
    id SERIAL PRIMARY KEY,
    operator_id INT NOT NULL REFERENCES operator(id) ON DELETE CASCADE,
    tariff_id INT NOT NULL REFERENCES tariff(id) ON DELETE CASCADE,
    c_type ChargeType NOT NULL,
    price DOUBLE PRECISION NOT NULL DEFAULT 0,
    blocking_fee_start BIGINT DEFAULT 0,
    blocking_fee DOUBLE PRECISION DEFAULT 0,
    day_of_week day_of_week[] NOT NULL DEFAULT '{monday,tuesday,wednesday,thursday,friday,saturday,sunday}',
    start_time TIME,
    end_time TIME,
    valid_from DATE,
    valid_until DATE,
    updated TIMESTAMPTZ DEFAULT now(),
    UNIQUE NULLS NOT DISTINCT (operator_id, tariff_id, c_type, day_of_week, start_time, end_time, valid_from, valid_until)
);

CREATE TABLE location_dynamic_price (
    location_id BIGINT NOT NULL REFERENCES charging_location(id) ON DELETE CASCADE,
    dynamic_price_id INT NOT NULL REFERENCES dynamic_charge_price(id) ON DELETE CASCADE,
    PRIMARY KEY (location_id, dynamic_price_id)
);
CREATE INDEX idx_ldp_price ON location_dynamic_price (dynamic_price_id);
