INSERT INTO impression_banner(
	banner_link, 
	platform
) VALUES (
	(select id from link_banner WHERE pub_id = $1),
	$2
);

