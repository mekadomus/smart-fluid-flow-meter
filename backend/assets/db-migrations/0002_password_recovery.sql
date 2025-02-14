CREATE TABLE password_recovery (
  token VARCHAR(100) NOT NULL,
  account_id VARCHAR(255) NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  recorded_at TIMESTAMP NOT NULL,
  PRIMARY KEY(token),
  CONSTRAINT fk_account_id FOREIGN KEY(account_id) REFERENCES account(id)
);
