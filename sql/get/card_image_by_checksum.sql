select file_path, mime_type, checksum
from tariff_image 
where checksum = $1 
ORDER BY updated desc limit 1