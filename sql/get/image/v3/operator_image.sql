select
    operator.pub_network as relation_id,
    checksum as blake3sum,
    image.updated as last_updated_date
from image
join operator on image.id = operator.image
where soft_delete = false
