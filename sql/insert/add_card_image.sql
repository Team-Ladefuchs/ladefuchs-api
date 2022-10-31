with inserted as (
    insert into tariff_image(file_path, checksum, mime_type, updated, soft_delete)
        values ($1, $2, $3, $4, false)
        on conflict(file_path) do update
            set
                checksum = excluded.checksum,
                mime_type = excluded.mime_type,
                updated = $4,
                soft_delete = false
        returning id
)

select id from inserted

union all

select id
from tariff_image
where file_path = $1
