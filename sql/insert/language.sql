WITH inserted as (
    INSERT INTO
    feed_language(name)
    VALUES
        ($1)
    ON CONFLICT DO NOTHING
    RETURNING ID
)
SELECT id FROM inserted

UNION ALL

SELECT id
FROM feed_language
WHERE name = $1