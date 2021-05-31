SELECT  f.id, f.title,
        f.description, f.subtitle,
        f.url, img.link as img,
        f.link_web, author.name as author,
        feed_language.name as language, f.last_modified,
        img.filename as img_cache
FROM
    feed f LEFT JOIN  author ON author.id = f.author_id
           LEFT JOIN feed_language ON feed_language.id = f.language
           LEFT JOIN img ON f.img_id = img.id
WHERE 
        f.status = 'online' AND 
        f.title = $1 OR
        (
            websearch_to_tsquery($1)::text <> '' AND
            f.search || author.search @@ to_tsquery(websearch_to_tsquery($1)::text || ':*')
        )
        
ORDER BY 
    ts_rank(f.search || author.search, to_tsquery(plainto_tsquery($1)::text || ':*')) DESC
LIMIT 50