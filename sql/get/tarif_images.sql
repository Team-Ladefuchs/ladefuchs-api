select 
     tarif.pub_tarif_id as tarif_identifier, 
    checksum, 
    mime_type,
    updated, 
    $1 || 'images/card/' || tarif_image.checksum as "url!"
from tarif_image 
    join tarif on tarif_image.id = tarif.image