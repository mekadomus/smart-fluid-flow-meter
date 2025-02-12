-- user is a reserved word in postgres, so we use account
CREATE TABLE account (
  id VARCHAR(255) NOT NULL,
  provider VARCHAR(30) NOT NULL,
  email VARCHAR(255) NOT NULL,
  password VARCHAR(100),
  name VARCHAR(255) NOT NULL,
  email_verified_at TIMESTAMP,
  recorded_at TIMESTAMP NOT NULL,
  PRIMARY KEY(id)
);

CREATE TABLE fluid_meter (
  id VARCHAR(255) NOT NULL,
  owner_id VARCHAR(255) NOT NULL,
  name VARCHAR(255) NOT NULL,
  status VARCHAR(255) NOT NULL,
  recorded_at TIMESTAMP NOT NULL,
  PRIMARY KEY(id),
  CONSTRAINT fk_owner_id FOREIGN KEY(owner_id) REFERENCES account(id)
);

CREATE TABLE measurement (
  id VARCHAR(255) NOT NULL,
  device_id VARCHAR(255) NOT NULL,
  measurement VARCHAR(255) NOT NULL,
  recorded_at TIMESTAMP NOT NULL,
  PRIMARY KEY(id),
  CONSTRAINT fk_device_id FOREIGN KEY(device_id) REFERENCES fluid_meter(id)
);

CREATE TABLE session_token (
  token VARCHAR(255) NOT NULL,
  account_id VARCHAR(255) NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  PRIMARY KEY(token),
  CONSTRAINT fk_account_id FOREIGN KEY(account_id) REFERENCES account(id)
);

CREATE TABLE email_verification (
  token VARCHAR(100) NOT NULL,
  account_id VARCHAR(255) NOT NULL,
  recorded_at TIMESTAMP NOT NULL,
  PRIMARY KEY(token),
  CONSTRAINT fk_account_id FOREIGN KEY(account_id) REFERENCES account(id)
);

CREATE INDEX idx_fluid_meter_name ON fluid_meter(name);
