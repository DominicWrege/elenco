Select id, name
FROM author
WHERE id = ANY 
    (   SELECT author_id 
        FROM feed 
        WHERE status = 'online'
    )
