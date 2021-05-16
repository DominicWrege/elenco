DROP FUNCTION author_tsvector CASCADE;

CREATE FUNCTION author_tsvector() RETURNS trigger AS $$
begin
    new.search := setweight(to_tsvector(coalesce(new.name,'')), 'A');
    return new;
end
$$ LANGUAGE plpgsql;

CREATE TRIGGER author_tsvector_update BEFORE INSERT OR UPDATE
    ON author FOR EACH ROW EXECUTE PROCEDURE author_tsvector();