-- Your SQL goes here
create table dengs (
	id SERIAL PRIMARY KEY,
	ts TIMESTAMP NOT NULL,
	user_id VARCHAR NOT NULL,
	successful BOOLEAN NOT NULL,
	days_first_deng BOOLEAN NOT NULL,
	users_first_deng BOOLEAN NOT NULL
)
