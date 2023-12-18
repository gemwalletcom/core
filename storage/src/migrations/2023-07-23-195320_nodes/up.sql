CREATE TABLE nodes (
  id SERIAL PRIMARY KEY,
  chain VARCHAR NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
  url VARCHAR NOT NULL,
  status VARCHAR NOT NULL,
  priority INTEGER NOT NULL,
  updated_at timestamp default current_timestamp,
  created_at timestamp default current_timestamp
)