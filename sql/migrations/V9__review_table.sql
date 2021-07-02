create type review_status as enum ('waiting', 'assigned', 'done');

CREATE TABLE review(
   status review_status DEFAULT 'waiting',
   reviewer integer references account(id) on delete cascade,
   modified timestamptz not null default CURRENT_TIMESTAMP,
   feed_id integer references feed(id) on delete cascade not null,
   id SERIAL,
   PRIMARY KEY (feed_id, id)
);


CREATE FUNCTION add_review() RETURNS trigger AS $$
begin
    INSERT INTO review (feed_id) VALUES (new.id);
    RETURN new;
end
$$ LANGUAGE plpgsql;

CREATE TRIGGER add_review_trigger AFTER INSERT
    ON feed FOR EACH ROW EXECUTE PROCEDURE add_review();