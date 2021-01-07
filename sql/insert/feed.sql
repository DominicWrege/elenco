INSERT INTO feed (
    submitter_id,
    author_id,
    title,
    description,
    img_id,
    subtitle,
    url,
    language,
    link_web
)
VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id