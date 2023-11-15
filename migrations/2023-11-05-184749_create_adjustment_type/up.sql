CREATE TABLE adjustment_type (
  id SERIAL PRIMARY KEY,
  description VARCHAR(255) NOT NULL,
  adjustment TINYINT SIGNED NOT NULL
);
