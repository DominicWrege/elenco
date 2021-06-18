SELECT id, content, a.username, a.id as username_id, feed_id
FROM comment 
    INNER JOIN account a on a.id = comment.user_id 
    INNER JOIN feed f on comment.feed_id = f.id
WHERE feed_id = $1