-- Add down migration script here

drop extension pg_trgm;

drop table if exists cpo_cache cascade;

drop FUNCTION if exists cpo_cache_tsvector;

drop trigger if exists cpo_cache_tsvector_update ON cpo_cache;

drop INDEX if exists cpo_cache_index;

