update tariff_image
    set file_path = $2
where file_path = $1
returning id