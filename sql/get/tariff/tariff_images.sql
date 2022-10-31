select 
    tariff.pub_tariff_id as tariff_identifier, 
    checksum, 
    mime_type,
    tariff.updated,
    slug_name as tariff_name,
    $1 || 'img/card/' || tariff_image.checksum as "url!"
from tariff_image 
    join tariff on tariff_image.id = tariff.image
where soft_delete = false
