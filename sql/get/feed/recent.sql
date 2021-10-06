SELECT  f.id,
        f.title,
        f.subtitle,
        author.name as author_name,
        f.submitted,
        img.filename as img,
        f.status
FROM
    review r JOIN feed f ON r.feed_id = f.id
             JOIN  author ON author.id = f.author_id
             JOIN img on f.img_id = img.id
WHERE f.status = 'online' AND r.status = 'done'
ORDER BY r.modified DESC, f.submitted DESC
LIMIT 20