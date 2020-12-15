SELECT username, email, id, password_hash, account_type as permission
FROM Account 
WHERE email = $1