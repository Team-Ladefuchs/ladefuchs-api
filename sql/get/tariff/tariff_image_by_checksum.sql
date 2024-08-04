select
    file_path,
    mime_type,
    checksum
from image
where checksum = $1 and soft_delete = false
order by updated desc limit 1
