SELECT id FROM feed WHERE title ILIKE $1 and status = 'online'

-- lowercase should workd