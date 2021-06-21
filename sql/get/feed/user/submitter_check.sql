Select DISTINCT submitter_id
FROM feed
WHERE feed.id = $1 and submitter_id = $2