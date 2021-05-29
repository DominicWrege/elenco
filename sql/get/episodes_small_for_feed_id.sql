SELECT  e.title, e.duration, e.url as url, published, explicit, media_url
FROM feed JOIN episode e on feed.id = e.feed_id
WHERE feed.id = $1
ORDER BY e.published DESC
LIMIT 100