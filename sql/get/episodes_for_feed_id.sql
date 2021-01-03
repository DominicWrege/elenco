SELECT  e.title,
        e.description,
        e.published,
        e.explicit,
        e.duration,
        e.show_notes,
        e.url as web_link,
        e.media_url,
        e.keywords
FROM feed JOIN episode e on feed.id = e.feed_id
WHERE feed.id = $1

