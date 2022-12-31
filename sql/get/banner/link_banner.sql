select 
  pub_id as id, 
  l.source,
  l.is_affiliate,
  replace(image_path, ' ', '') as "image!", 
  frequency, 
  updated
from link_banner 
    join link l on l.id = link_banner.link_id
where expiration > now() 
