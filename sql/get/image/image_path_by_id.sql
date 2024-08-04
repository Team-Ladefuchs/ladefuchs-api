select file_path
from image
where id = $1
limit 1
