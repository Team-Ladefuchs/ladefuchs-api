select checksum
from image
where id = $1
limit 1
