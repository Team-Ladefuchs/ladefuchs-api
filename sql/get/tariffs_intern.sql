select 
    tariff.slug_name, 
    tariff.pub_tariff_id as id,
    tariff.url,
    ti.file_path as "file_path?",
    m.name as msp_name,
    ti.updated as "updated?",
    ti.checksum as "checksum?",
    tariff.internal_name
from tariff left join tariff_image ti on tariff.image = ti.id
     join msp m on m.id = tariff.msp_id
where ti.soft_delete = false
order by ti.updated DESC NULLS LAST, tariff.slug_name
