SELECT f.id, 
       f.title,
       f.subtitle,
       author.name as author_name,
       img.filename as img,
       f.status,
       s.created as submitted
FROM feed f
       LEFT JOIN subscription s ON s.feed_id = f.id
       LEFT JOIN author ON author.id = f.author_id
       LEFT JOIN feed_language ON feed_language.id = f.language
       LEFT JOIN img ON f.img_id = img.id
WHERE f.status = 'online' AND s.user_id = $1 
ORDER BY f.title