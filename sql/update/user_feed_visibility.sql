UPDATE feed 
SET status = $1
WHERE id = $2 
AND status != 'blocked' AND status != 'queued' 
AND submitter_id = $3