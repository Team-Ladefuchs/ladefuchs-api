select id 
from tariff_image
where is_ad_hoc = true and soft_delete = false
limit 1
