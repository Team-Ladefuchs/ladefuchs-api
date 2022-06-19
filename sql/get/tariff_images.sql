select 
    tariff.pub_tariff_id as tariff_identifier, 
    checksum, 
    mime_type,
    updated,
    slug_name as tariff_name,
    $1 || 'images/card/' || tariff_image.checksum as "url!"
from tariff_image 
    join tariff on tariff_image.id = tariff.image