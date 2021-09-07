SELECT  f.title, author.name as author_name
FROM
    feed f LEFT JOIN author ON author.id = f.author_id
WHERE
    f.status = 'online' AND
    f.search || author.search @@ to_tsquery(websearch_to_tsquery($1)::text || ':*')
LIMIT 10