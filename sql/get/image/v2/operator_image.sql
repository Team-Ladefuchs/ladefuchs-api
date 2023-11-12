select 
    operator.pub_network as cpo_identifier, 
    checksum, 
    mime_type,
    image.updated,
    slug_name as cpo_name,
    $1 || 'img/cpo/' || image.checksum as "url!"
from image 
    join operator on image.id = operator.image
where soft_delete = false
