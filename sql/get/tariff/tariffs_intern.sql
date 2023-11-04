select
    tariff.slug_name,
	tariff.relationship_id,
    tariff.id,
    tariff.url,
    ti.file_path as "file_path?",
    monthly_fee,
    tariff.provider_name,
    GREATEST(ti.updated, tariff.updated) as "updated!",
    ti.checksum as "checksum?",
    tariff.internal_name,
	tariff.note,
	tariff.override_standard,
	tariff.standard,
	tariff.provider_customer_only,
	tariff.hide
from tariff left join image ti on tariff.image = ti.id
order by ti.updated DESC NULLS LAST, tariff.slug_name
