insert into msp(msp_id, name) values ($1, $2) 
on conflict(name) DO update set name = excluded.name, msp_id = excluded.msp_id
RETURNING id