select username, password_hash as password
from admin_user 
where username = $1