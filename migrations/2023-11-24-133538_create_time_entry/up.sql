CREATE TABLE time_entry (
  id SERIAL PRIMARY KEY,
  time SMALLINT UNSIGNED NOT NULL,
  created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);