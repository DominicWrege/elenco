SELECT categories.id, categories.description, categories.subcategories
FROM
    feed_category JOIN categories ON categories.id = feed_category.category_id
WHERE feed_id = $1