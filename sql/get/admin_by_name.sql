select
    id,
    username,
    password_hash
from admin_user
where username = $1
