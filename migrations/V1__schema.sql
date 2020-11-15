create type permission as enum ('user', 'admin');

create table account (
    id serial primary key,
    account_name text unique not null,
    password_hash text not null,
    email text unique not null,
    account_type permission not null default 'user' 
);

create table feed (
    id serial primary key,
    account serial references account(id) on delete cascade not null,
    title text unique not null,
    img_url text unique,
    description text not null,
    link text unique not null,
    author text not null
);