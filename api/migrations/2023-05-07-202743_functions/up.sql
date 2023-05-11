-- Your SQL goes here
CREATE TABLE functions (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  arity INTEGER NOT NULL,
  name VARCHAR NOT NULL,
  uri VARCHAR NOT NULL,
  user_uri VARCHAR NOT NULL,
  signature JSONB NOT NULL
);

CREATE TABLE invoke_requests (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  function_id INTEGER NOT NULL REFERENCES functions(id),
  user_addr VARCHAR NOT NULL,
  payload JSONB
);
