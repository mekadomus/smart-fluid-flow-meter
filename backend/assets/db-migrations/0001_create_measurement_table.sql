CREATE TABLE measurement (
  id INT NOT NULL AUTO_INCREMENT,
  device_id VARCHAR(255) NOT NULL,
  measurement VARCHAR(255) NOT NULL,
  recorded_at DATETIME NOT NULL,
  PRIMARY KEY(id)
);
