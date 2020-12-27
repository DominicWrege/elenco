SELECT c1.id, c1.description as description, json_agg(json_build_object('id', c2.id, 'description', c2.description)) as subcategorys
FROM category c1 LEFT JOIN category c2 ON c1.id = c2.parent_id
WHERE c1.id = $1 AND c2.id IS NOT NULL
GROUP BY c1.description, c1.id