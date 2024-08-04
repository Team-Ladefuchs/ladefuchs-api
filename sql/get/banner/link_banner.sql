select
    pub_id as id,
    l.source,
    l.is_affiliate,
    replace(i.file_path, ' ', '') as "image!",
    frequency,
    i.checksum,
    greatest(link_banner.updated, i.updated) as "updated!"
from link_banner
join link l on l.id = link_banner.link_id
join image i on i.id = link_banner.image
where starts <= now() and expiration >= now()
