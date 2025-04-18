create schema if not exists eco_movement;

create table if not exists eco_movement.operator
(
    id UUID primary key,
    name TEXT not null,
    website TEXT,
    ema_id TEXT [] not null default array[]::TEXT []
);

create type eco_movement.powertype as enum ('ac3_phase', 'ac1_phase', 'dc');

create table if not exists eco_movement.connector
(
    id TEXT not null,
    evse_uid TEXT not null,
    primary key (id, evse_uid),
    power_type eco_movement.POWERTYPE not null,
    max_power INT not null
);

create type eco_movement.locationtype as enum (
    'on_street',
    'parking_garage',
    'underground_garage',
    'parking_lot',
    'other',
    'unknown'
);

create table if not exists eco_movement.location (
    id UUID primary key,
    value JSON not null,
    country CHAR(3) not null default 'DEU',
    type eco_movement.LOCATIONTYPE not null,
    operator_id UUID not null references eco_movement.operator (
        id
    ) on delete cascade
);

create type eco_movement.tarifftype as enum (
    'msp', 'adhoc', 'cpo_subscription'
);

create table eco_movement.tariff (
    id INT generated always as identity primary key,
    name TEXT not null,
    description TEXT,
    subscription_type TEXT,
    type eco_movement.TARIFFTYPE not null,
    subscription_fee_excl_vat TEXT not null,
    currency CHAR(3) not null,
    provider_name TEXT not null,
    unique (name, provider_name)
);

create table if not exists eco_movement.price (
    id CHAR(64) primary key,
    provider_name TEXT not null,
    tariff_id INT not null references eco_movement.tariff (
        id
    ) on delete cascade,
    elements JSON not null
);

create table if not exists eco_movement.connector_price (
    location_id UUID not null references eco_movement.location (
        id
    ) on delete cascade,
    pricing_id CHAR(64) not null references eco_movement.price (
        id
    ) on delete cascade,
    evse_uid TEXT not null,
    connector_id TEXT not null,
    primary key (location_id, evse_uid, connector_id),
    foreign key (connector_id, evse_uid)
    references eco_movement.connector (id, evse_uid) on delete cascade
);
