WITH inserted as (
    INSERT INTO
    category(description, parent_id)
    VALUES
        ($1,$2)
    ON CONFLICT DO NOTHING
    RETURNING ID
)
SELECT id FROM inserted

UNION ALL

SELECT id
FROM category
WHERE description = $1