UPDATE feed SET 
    status = $1,
    last_modified = CURRENT_TIMESTAMP
    WHERE id = $2 AND status = 'queued'