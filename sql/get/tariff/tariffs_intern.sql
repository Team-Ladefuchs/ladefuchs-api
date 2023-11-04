select
    tariff.slug_name,
	tariff.relationship_id,
    tariff.id,
    tariff.url,
    image.file_path as "file_path?",
    tariff.monthly_fee,
    tariff.provider_name,
    GREATEST(image.updated, tariff.updated) as "updated!",
    image.checksum as "checksum?",
    tariff.internal_name,
	tariff.note,
	tariff.override_standard,
	tariff.standard,
	tariff.provider_customer_only,
	tariff.hide,
	image as image_id
from tariff left join image on tariff.image = image.id
order by image.updated DESC NULLS LAST, tariff.slug_name
