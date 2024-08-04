INSERT INTO impression_banner (
    banner_link,
    platform
) VALUES (
    (SELECT id FROM link_banner WHERE pub_id = $1),
    $2
);
