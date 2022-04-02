-- Add up migration script here
alter table cpo add column power_AC int4 not null default 22 check ( power_AC >= 0);
alter table cpo add column power_DC int4 not null default 50 check ( power_DC >= 0);
alter table cpo add column expect_AC int4 not null default 3 check ( expect_AC >= 0);
alter table cpo add column expect_DC int4 not null default 3 check ( expect_DC >= 0);

CREATE PROCEDURE change_cpo()
    LANGUAGE plpgsql
AS $$
DECLARE
row RECORD;
    BEGIN
        FOR row IN SELECT network, extra FROM cpo
            LOOP
                update cpo
                    set power_AC = (extra ->> 'powerAC')::int4,
                        power_DC = (extra ->> 'powerDC')::int4,
                        expect_AC = (extra ->> 'expectAC')::int4,
                        expect_DC = (extra ->> 'expectDC')::int4
                    where cpo.network = network;
            END LOOP;
    END;
$$;


call change_cpo();

drop procedure if exists change_cpo;

alter table cpo drop column extra;