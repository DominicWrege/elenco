
create type permission as enum ('user', 'admin');

create table account (
    id serial primary key,
    account_name text unique not null,
    password_hash text not null,
    email text unique not null,
    account_type permission default 'user'
);

