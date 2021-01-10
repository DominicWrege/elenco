Select id, description
from feed_category join category c on feed_category.category_id = c.id
WHERE feed_id = $1 and parent_id = $2