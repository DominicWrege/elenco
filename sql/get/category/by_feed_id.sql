Select id, description
from feed_category join category c on feed_category.category_id = c.id
WHERE c.parent_id is null and feed_id = $1