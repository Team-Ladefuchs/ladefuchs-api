-- Add up migration script here
alter table cpo add column supported_types chargetype[] not null default array['AC', 'DC']::chargetype[];

update cpo
set supported_types =  case
                           when expect_ac > 0 and expect_dc > 0 then array['AC', 'DC']::chargetype[]
                           when expect_ac > 0 then array['AC']::chargetype[]
                           when expect_dc > 0 then array['DC']::chargetype[]
                           else array ['AC']::chargetype[]
    end
where cpo.id = id;

alter table cpo drop column expect_ac;
alter table cpo drop column expect_dc;
