select username, password_hash as password
from admin 
where username = $1