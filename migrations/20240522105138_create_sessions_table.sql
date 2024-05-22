CREATE TABLE sessions (
  token VARCHAR(1024),
  username VARCHAR(128) REFERENCES users(username)
);
