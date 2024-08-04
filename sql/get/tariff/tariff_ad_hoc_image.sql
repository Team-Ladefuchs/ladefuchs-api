select id
from image
where is_ad_hoc = true and soft_delete = false
limit 1
