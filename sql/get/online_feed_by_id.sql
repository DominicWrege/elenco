        SELECT f.id, f.title, f.description, f.subtitle, f.url, img.link as img,
       f.link_web, author.name as author,
       feed_language.name as language, f.last_modified
        FROM
            feed f LEFT JOIN  author ON author.id = f.author_id
                LEFT JOIN feed_language ON feed_language.id = f.language
                    LEFT JOIN img ON f.img_id = img.id WHERE f.id = $1 AND f.status = 'online'