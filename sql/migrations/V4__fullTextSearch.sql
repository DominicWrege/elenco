---feed
ALTER TABLE feed
    add column search tsvector NOT NULL default '';

Update feed
    set search =
        setweight(to_tsvector(coalesce(title,'')), 'A') ||
        setweight(to_tsvector(coalesce(description,'')), 'B') ||
        setweight(to_tsvector(coalesce(subtitle,'')), 'C');


CREATE FUNCTION feed_tsvector() RETURNS trigger AS $$
begin
    new.search :=
        setweight(to_tsvector(coalesce(new.title,'')), 'A') ||
        setweight(to_tsvector(coalesce(new.description,'')), 'B') ||
        setweight(to_tsvector(coalesce(new.subtitle,'')), 'C');
    return new;
end
$$ LANGUAGE plpgsql;

CREATE TRIGGER feed_tsvector_update BEFORE INSERT OR UPDATE
    ON feed FOR EACH ROW EXECUTE PROCEDURE feed_tsvector();

CREATE INDEX search_index ON feed USING GIN(search);
---feed

---author
ALTER TABLE author
    add column search tsvector NOT NULL default '';
Update author
    set search = setweight(to_tsvector(coalesce(name,'')), 'A');

CREATE FUNCTION author_tsvector() RETURNS trigger AS $$
begin
    new.sarch := setweight(to_tsvector(coalesce(new.author,'')), 'A');
    return new;
end
$$ LANGUAGE plpgsql;

CREATE TRIGGER author_tsvector_update BEFORE INSERT OR UPDATE
    ON author FOR EACH ROW EXECUTE PROCEDURE author_tsvector();

CREATE INDEX author_index ON author USING GIN(search);
---author