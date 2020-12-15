SELECT f.id, f.title, f.description, f.url, f.submitted, f.img_path,
    f. link_web, f.status::text, a.name as author_name, 
    fl.name as language, ac.username, f.last_modified
FROM author a , feed f, feed_language fl, account ac
WHERE 
    f.submitter_id = $1 AND
    a.id = f.author_id AND
    f.submitter_id = ac.id AND
    fl.id = f.language
