CREATE TYPE vehicle as ENUM('TRAIN', 'BUS');
CREATE TYPE availability_status as ENUM('AVAILABLE', 'UNAVAILABLE');

CREATE TABLE schedules (
  id INT NOT NULL,
  line INT NOT NULL,
  from_place_id INT NOT NULL,
  to_place_id INT NOT NULL,
  type vehicle DEFAULT NULL,
  departure_time time DEFAULT NULL,
  arrival_time time DEFAULT NULL,
  distance INT DEFAULT NULL,
  speed INT DEFAULT NULL,
  status availability_status DEFAULT 'AVAILABLE'
);
