UPDATE review SET 
    modified = CURRENT_TIMESTAMP,
    reviewer = $1
WHERE id= $2