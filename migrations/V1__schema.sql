create type permission as enum ('user', 'admin');

create table account (
    id serial primary key,
    username text not null check ( username <> ''),
    password_hash text not null check ( password_hash <> '' ),
    email text unique not null check ( email <> '' ),
    created timestamp not null default CURRENT_TIMESTAMP,
    account_type permission not null default 'user' 
);
create type feed_status as enum ('online', 'offline', 'blocked', 'queued');

create table author (
    id serial primary key,
    name text unique not null check ( name <> '' )
);

create table feed_language (
   id serial primary key,
   name text unique not null check ( name <> '' )
);

create table feed (
    id serial primary key,
    submitter_id integer references account(id) on delete cascade not null,
    author_id integer references author(id) on delete cascade not null,
    title text unique not null check ( title <> '' ),
    img_path text unique check ( img_path <> '' ),
    description text not null check ( description <> '' ),
    subtitle text check ( subtitle <> '' ),
    url text unique not null check ( url <> '' ),
    language integer references feed_language(id),
    copyright text check ( copyright <> '' ),
    status feed_status not null default 'queued',
    submitted timestamp not null default CURRENT_TIMESTAMP not null,
    last_modified timestamp not null default CURRENT_TIMESTAMP not null
);

create table category (
    id serial primary key,
    description text unique not null check( description <> '' )
);
create table feed_category (
    feed_id integer references feed(id) not null,
    category_id integer references category(id) not null,
    primary key (feed_id, category_id)
);

create table episode (
    id bigserial primary key,
    title text unique not null check ( title <> '' ),
    description text not null check ( description <> '' ),
    published timestamp not null,
    explicit bool default false,
    keywords text[],
    duration integer check ( duration > 0 ),
    show_notes text check ( show_notes <> '' ),
    url text check ( url <> '' ),
    media_url text not null check ( media_url <> '' ),
    feed_id integer references feed(id) on delete cascade not null
);