create table img (
       id serial primary key,
       link text check ( link <> '' ) not null,
       filename text check ( filename <> '' ) not null,
       hash text unique check ( hash <> '' ) not null
);
ALTER TABLE feed DROP COLUMN img_path;
ALTER TABLE feed ADD COLUMN img_id integer references img(id) on delete cascade;