select
    pub_id as id,
    l.source,
    l.is_affiliate,
    frequency,
    i.checksum,
    replace(i.file_path, ' ', '') as "image!",
    greatest(link_banner.updated, i.updated) as "updated!"
from link_banner
inner join link as l on link_banner.link_id = l.id
inner join image as i on link_banner.image = i.id
where
    case
        when
            link_banner.impression = 0
            then starts <= now() and expiration >= now()
        else
            starts <= now()
            and expiration >= now()
            and impression >= (
                select count(id) from impression_banner
                where banner_link = link_banner.id
            )
    end
