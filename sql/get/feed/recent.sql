SELECT  f.id,
        f.title,
        f.subtitle,
        author.name as author_name,
        f.submitted,
        img.filename as img,
        f.status
FROM subscription s
    RIGHT JOIN feed f on s.feed_id = f.id
    JOIN author on f.author_id = author.id
    JOIN img on f.img_id = img.id
WHERE f.status = 'online'
ORDER BY f.submitted DESC
LIMIT 15;