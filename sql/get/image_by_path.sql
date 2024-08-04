select id
from image
where file_path = $1
limit 1
