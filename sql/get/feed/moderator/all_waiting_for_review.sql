SELECT f.id,
       r.id as review_id,
       f.title,
       f.url,
       f.submitted,
       f.link_web,
       f.status,
       author.name as author_name,
       account.username
FROM
    review r LEFT JOIN feed f ON r.feed_id = f.id
    LEFT JOIN  author ON author.id = f.author_id
    LEFT JOIN feed_language ON feed_language.id = f.language
    LEFT JOIN account ON account.id = f.submitter_id
WHERE r.reviewer IS NULL AND r.status = 'waiting'
ORDER BY submitted DESC
LIMIT 100