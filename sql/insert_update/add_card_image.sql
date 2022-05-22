insert into tarif_image(tarif_id, file_path, checksum, filename)
VALUES ($1, $2, $3, $4)
    on conflict(filename, filename) do update
        set
            checksum = excluded.checksum,
            filename = excluded.filename,
            file_path = excluded.file_path,
            updated = now()