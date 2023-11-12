select
	pub_tariff_id as relation_id,
	checksum as blake3sum,
	image.updated as last_updated_date
from image
	join tariff on image.id = tariff.image
where soft_delete = false
