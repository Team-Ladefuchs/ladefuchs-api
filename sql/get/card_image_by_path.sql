select id
from tariff_image 
where file_path = $1 
limit 1