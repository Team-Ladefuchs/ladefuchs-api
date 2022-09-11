update tariff_image
    set soft_delete = true
where id = $1 
