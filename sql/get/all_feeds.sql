    SELECT f.id, f.title, f.description, f.url, f.submitted, img.link as img_cache,
           f.link_web, f.status::text, author.name as author_name,
           feed_language.name as language, account.username, f.last_modified,
           f.subtitle, f.search
    FROM
        feed f LEFT JOIN  author ON author.id = f.author_id
                  LEFT JOIN feed_language ON feed_language.id = f.language
                  LEFT JOIN account ON account.id = f.submitter_id
                  LEFT JOIN img ON f.img_id = img.id;