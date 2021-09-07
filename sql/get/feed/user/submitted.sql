SELECT  f.id,
        f.title, 
        f.description,
        f.submitted, 
        img.filename as img, 
        author.name as author_name,
        f.status,
        f.subtitle
FROM feed f JOIN author ON author.id = f.author_id
            JOIN img ON f.img_id = img.id
WHERE f.submitter_id = $1
ORDER BY f.status, f.submitted