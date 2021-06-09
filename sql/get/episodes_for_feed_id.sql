SELECT  e.title,
        e.description,
        e.published,
        e.explicit,
        e.duration,
        e.show_notes,
        e.url as web_link,
        e.media_url,
        e.keywords,
        e.media_length,
        e.mime_type,
        e.guid
FROM feed JOIN episode e on feed.id = e.feed_id
WHERE feed.id = $1
ORDER BY e.published DESC
LIMIT 100