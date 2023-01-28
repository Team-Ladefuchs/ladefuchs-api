update image
    set file_path = $2, soft_delete = false
where file_path = $1
returning id
