select
    tariff.slug_name,
	tariff.relationship_id,
    tariff.id,
    tariff.url,
    ti.file_path as "file_path?",
    monthly_fee,
    tariff.provider_name as msp_name,
    GREATEST(ti.updated, tariff.updated) as "updated!",
    ti.checksum as "checksum?",
    tariff.internal_name,
	tariff.note,
	tariff.standard as ,
	    CASE 
        WHEN tariff.override_standard = true THEN tariff.override_standard
        ELSE tariff.standard
    END as "is_enabled!"
    CASE WHEN EXISTS (SELECT charge_price.cpo_id from charge_price where charge_price.tariff_id = tariff.id)
             then true
         else false
        END as "visible!"
from tariff left join image ti on tariff.image = ti.id
order by ti.updated DESC NULLS LAST, tariff.slug_name
