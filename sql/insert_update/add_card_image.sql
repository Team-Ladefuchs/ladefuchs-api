with inserted as (
    insert into tarif_image(file_path, checksum, mime_type)
        values ($1, $2, $3)
        on conflict(file_path) do update
            set
                checksum = excluded.checksum,
                mime_type = excluded.mime_type,
                updated = now()
        returning id
)

select id from inserted

union all

select id
from tarif_image
where file_path = $1