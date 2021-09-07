SELECT comment.id, 
       content, 
       a.username, 
       a.id as user_id, 
       feed_id,
       comment.created  
FROM comment 
    JOIN account a on a.id = comment.user_id 
    JOIN feed f on comment.feed_id = f.id
WHERE feed_id = $1
ORDER BY comment.created DESC