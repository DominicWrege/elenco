select * from AllFeeds 
WHERE status = 'queued'
ORDER BY submitted DESC
LIMIT 30