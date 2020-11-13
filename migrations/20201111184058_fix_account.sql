-- Add migration script here


alter table account 
ALTER COLUMN account_type SET NOT NULL;