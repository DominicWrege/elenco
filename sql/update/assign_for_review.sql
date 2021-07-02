UPDATE review SET 
    modified = CURRENT_TIMESTAMP,
    reviewer = $1
WHERE feed_id = $2