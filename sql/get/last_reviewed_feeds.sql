select * from AllFeeds
WHERE status != 'queued' AND status != 'offline'
ORDER BY AllFeeds.last_modified DESC
LIMIT 10;