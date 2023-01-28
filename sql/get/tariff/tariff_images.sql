select 
    tariff.pub_tariff_id as tariff_identifier, 
    checksum, 
    mime_type,
    image.updated,
    slug_name as tariff_name,
    $1 || 'img/card/' || image.checksum as "url!"
from image 
    join tariff on image.id = tariff.image
where soft_delete = false
