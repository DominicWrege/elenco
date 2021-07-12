SELECT status 
FROM feed
WHERE submitter_id = $1 AND id = $2 AND (status = 'online' or status = 'offline');
