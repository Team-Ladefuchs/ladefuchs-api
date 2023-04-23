select link_banner.id, image.file_path
from link_banner join image on link_banner.image = image.id
where pub_id = $1

