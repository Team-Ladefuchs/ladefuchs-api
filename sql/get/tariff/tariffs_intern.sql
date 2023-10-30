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
	case
        when tariff.override_standard = true THEN tariff.override_standard
        else tariff.standard
    end as "is_enabled!",
    case when EXISTS (SELECT charge_price.cpo_id from charge_price where charge_price.tariff_id = tariff.id)
             then true
         else false
    end as "standard!"
from tariff left join image ti on tariff.image = ti.id
order by ti.updated DESC NULLS LAST, tariff.slug_name
