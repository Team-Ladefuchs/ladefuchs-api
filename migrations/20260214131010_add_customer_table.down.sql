ALTER TABLE link_banner
	DROP COLUMN customer_id,
	ADD COLUMN impression INT NOT NULL DEFAULT 0;

DROP TABLE IF EXISTS customer;
