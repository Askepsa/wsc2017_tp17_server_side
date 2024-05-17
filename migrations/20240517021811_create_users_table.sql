CREATE TYPE role as ENUM('ADMIN', 'USER');

CREATE TABLE users (
  role role NOT NULL,
  username VARCHAR(128) PRIMARY KEY,
  password VARCHAR(256)
);
