SELECT f.id, f.title, 
        f.url, f.submitted, 
        f.link_web, f.status, author.name as author_name,
        feed_language.name as language, 
        account.username, 
        f.last_modified
FROM
    feed f LEFT JOIN  author ON author.id = f.author_id
        LEFT JOIN feed_language ON feed_language.id = f.language
        LEFT JOIN account ON account.id = f.submitter_id
WHERE status = 'queued'
ORDER BY submitted DESC
LIMIT 30