INSERT INTO  
    episode(title, description, published, explicit, keywords, 
            duration, show_notes, url, media_url, feed_id, guid, 
            media_length, mime_type
            )
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)