SELECT  f.id, f.title,
        f.description, f.subtitle,
        f.url, img.link as img,
        f.link_web, author.name as author_name,
        feed_language.name as language,
        f.submitted,
        img.filename as img_cache
FROM
    feed f
        JOIN  author ON author.id = f.author_id
        JOIN feed_language ON feed_language.id = f.language
        JOIN img ON f.img_id = img.id
WHERE f.id = $1