WITH inserted as (
    INSERT INTO
        author(name)
    VALUES
        ($1)
    ON CONFLICT DO NOTHING
    RETURNING ID
)
SELECT id FROM inserted

UNION ALL

SELECT id
FROM author
WHERE name = $1