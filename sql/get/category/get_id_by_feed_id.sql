SELECT category_id FROM feed_category
WHERE feed_id = $1
ORDER BY RANDOM()
LIMIT 1