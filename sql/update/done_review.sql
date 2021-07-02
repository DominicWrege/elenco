UPDATE review SET 
    status = 'done',
    modified = CURRENT_TIMESTAMP
    WHERE feed_id = $1 AND status = 'assigned'