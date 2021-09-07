
create table subscription(
    user_id integer references account(id) on delete cascade not null,
    created timestamptz not null default CURRENT_TIMESTAMP,
    feed_id integer references feed(id) on delete cascade not null,
    primary key (user_id, feed_id)
);

create table comment(
    id serial primary key,
    content text not null check (content <> ''),
    created timestamptz not null default CURRENT_TIMESTAMP,
    feed_id integer references feed(id) on delete cascade not null,
    user_id integer references account(id) not null
);