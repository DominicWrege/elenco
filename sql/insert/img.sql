WITH inserted as (
    INSERT INTO
    img(link, hash, filename)
    VALUES
        ($1, $2, $3)
    ON CONFLICT DO NOTHING
    RETURNING ID
)
SELECT id FROM inserted

UNION

SELECT id
FROM img
WHERE hash = $2