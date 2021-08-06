SELECT img.filename as img, f.id as feed_id
FROM feed f JOIN img ON f.img_id = img.id
WHERE f.title = $1 AND f.status = 'online'
LIMIT 1