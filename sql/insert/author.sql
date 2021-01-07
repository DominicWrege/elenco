WITH inserted as (
    INSERT INTO
        author(name, search)
    VALUES
        ($1, ''::tsvector)
    ON CONFLICT DO NOTHING
    RETURNING ID
)
SELECT id FROM inserted

UNION ALL

SELECT id
FROM author
WHERE name = $1