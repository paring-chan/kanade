-- column drops are safe for now because I not deployed this anywhere

ALTER TABLE user_forge DROP COLUMN access_token;
ALTER TABLE user_forge DROP COLUMN refresh_token;

ALTER TABLE user_forge ADD COLUMN access_token BYTEA NOT NULL;
ALTER TABLE user_forge ADD COLUMN refresh_token BYTEA NOT NULL;

ALTER TABLE repo ADD COLUMN created_by UUID NOT NULL REFERENCES "user" (id) ON DELETE RESTRICT;
