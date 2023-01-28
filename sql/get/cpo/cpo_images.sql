select 
    cpo.pub_network as cpo_identifier, 
    checksum, 
    mime_type,
    image.updated,
    slug_name as cpo_name,
    $1 || 'img/cpo/' || image.checksum as "url!"
from image 
    join cpo on image.id = cpo.image
where soft_delete = false
