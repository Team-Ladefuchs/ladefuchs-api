select
    link_banner.pub_id as relation_id,
    checksum as blake3sum,
    image.updated as last_updated_date
from image
         join link_banner on image.id = image
where soft_delete = false
