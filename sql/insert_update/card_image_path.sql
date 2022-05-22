update tarif_image
    set file_path = $2, filename = $3 
where file_path = $1 
returning tarif_id