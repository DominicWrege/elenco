SELECT
    e.id,
    e.title,
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
FROM episode e
WHERE e.feed_id = $1 AND e.id > $2
ORDER BY e.published DESC
LIMIT $3