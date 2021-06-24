SELECT  c1.id, c1.description, 
        COALESCE(json_agg(json_build_object('id', c2.id, 'description', c2.description)) 
            FILTER (WHERE c2.id IS NOT NULL), '[]') as subcategories
FROM category c1 LEFT JOIN category c2 ON c1.id = c2.parent_id
WHERE c1.parent_id IS NULL AND c1.description = $1
GROUP BY c1.id, c1.description
ORDER BY c1.description