-- Add up migration script here
CREATE OR REPLACE procedure update_image_file_path()
    AS $$
BEGIN
    UPDATE image
    SET file_path = (SELECT substring(file_path FROM 'images/(?:cards|banners|cpos)/.*'))
    WHERE file_path LIKE '/%'
      AND COALESCE((SELECT substring(file_path FROM 'images/(?:cards|banners|cpos)/.*')), '') <> '';
END;
$$ LANGUAGE plpgsql;
call update_image_file_path();
drop procedure if exists update_image_file_path;
