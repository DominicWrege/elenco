SELECT f.id,
       r.id as review_id,
       f.title,
       f.url,
       f.submitted,
       r.modified,
       f.link_web,
       f.status,
       author.name as author_name,
       account.username,
       (Select username From account WHERE r.reviewer = account.id) reviewer_name
FROM
    review r JOIN feed f ON r.feed_id = f.id
            JOIN  author ON author.id = f.author_id
            JOIN feed_language ON feed_language.id = f.language
            JOIN account ON account.id = f.submitter_id
WHERE r.reviewer IS NULL AND r.status = 'waiting'
ORDER BY submitted DESC
LIMIT 100