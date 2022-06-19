update tariff_image
    set 
        file_path = $2,
        checksum = $3,
        mime_type = $4,
        updated = $5
where id = $1