select file_path, mime_type, checksum
from tarif_image 
where checksum = $1 
ORDER BY updated desc limit 1