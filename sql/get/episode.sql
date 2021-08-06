SELECT
    id,
    title,
    description,
    published,
    explicit,
    duration,
    show_notes,
    url as web_link,
    media_url,
    keywords,
    media_length,
    mime_type,
    guid
FROM episode 
WHERE id = $1