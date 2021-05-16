SELECT f.id, f.title, f.url, f.submitted, img.link as img_cache,
        f.link_web, f.status, author.name as author_name,
        feed_language.name as language, account.username, f.last_modified
FROM feed f LEFT JOIN  author ON author.id = f.author_id
                LEFT JOIN feed_language ON feed_language.id = f.language
                LEFT JOIN account ON account.id = f.submitter_id
                LEFT JOIN img ON f.img_id = img.id
WHERE status != 'queued' AND status != 'offline'
ORDER BY f.last_modified DESC
LIMIT 15;