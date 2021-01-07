DROP FUNCTION author_tsvector CASCADE;

CREATE FUNCTION author_tsvector() RETURNS trigger AS $$
begin
    new.sarch := setweight(to_tsvector(coalesce(new.name,'')), 'A');
    return new;
end
$$ LANGUAGE plpgsql;