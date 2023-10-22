-- Add up migration script here
drop table if exists cpo_cache cascade;
drop trigger if exists cpo_cache_tsvector_update on cpo_cache;
drop function if exists cpo_cache_tsvector;

alter table if exists cpo rename to operator;

alter table operator add column if not exists url text,
add if not exists search tsvector NOT NULL default '',
add column if not exists ccs_plug_count integer NOT NULL DEFAULT 0,
add column if not exists type2_plug_count integer NOT NULL DEFAULT 0;

CREATE OR REPLACE FUNCTION operator_search_tsvector() RETURNS trigger AS $$
begin
    new.search := setweight(to_tsvector(coalesce(new.slug_name,'')), 'A');
    return new;
end
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER operator_search_tsvector_update BEFORE INSERT OR UPDATE
    ON operator FOR EACH ROW EXECUTE PROCEDURE operator_search_tsvector();

UPDATE operator
SET search = setweight(to_tsvector(coalesce(slug_name, '')), 'A');

create index if not exists operator_cache_index ON operator USING GIN(search);



