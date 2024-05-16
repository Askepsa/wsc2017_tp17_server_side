CREATE TYPE place_category as ENUM('Attraction', 'Restaurant');

CREATE TABLE places (
  id INT NOT NULL,
  name VARCHAR(100) DEFAULT NULL,
  latitude FLOAT DEFAULT NULL,
  longitude FLOAT DEFAULT NULL,
  x INT DEFAULT NULL,
  y INT DEFAULT NULL,
  type place_category DEFAULT NULL,
  image_path VARCHAR(50) DEFAULT NULL,
  open_time time DEFAULT NULL,
  close_time time DEFAULT NULL,
  description text
);

