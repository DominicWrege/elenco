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
GROUP BY f.id, f.title, f.subtitle, author_name, img, f.status
ORDER BY count(s.feed_id) DESC, (select count(feed_id) FROM comment where feed_id = comment.feed_id ) DESC
LIMIT 25;