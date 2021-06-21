SELECT  f.id,
        f.title,
        f.description,
        f.submitted,
        img.filename as img,
        author.name as author_name,
        f.status,
        f.subtitle,
        f.submitted
FROM feed f LEFT JOIN author ON author.id = f.author_id
            LEFT JOIN img ON f.img_id = img.id
            LEFT JOIN feed_category fc on f.id = fc.feed_id
WHERE f.status = 'online' AND fc.category_id = $1 AND f.id != $2
ORDER BY RANDOM()
LIMIT 5