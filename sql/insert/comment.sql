WITH inserted AS (
    INSERT INTO comment(content, user_id, feed_id)
    VALUES ($1, $2, $3 ) RETURNING id, user_id, feed_id, created, content
)
SELECT inserted.id, content, account.username, inserted.user_id, feed_id, inserted.created
FROM inserted, account
WHERE account.id = inserted.user_id;