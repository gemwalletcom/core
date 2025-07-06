CREATE TABLE nodes (
  id SERIAL PRIMARY KEY,
  chain VARCHAR NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
  url VARCHAR NOT NULL,
  node_type VARCHAR NOT NULL default 'default',
  status VARCHAR NOT NULL default 'active',
  priority INTEGER NOT NULL default 10,
  updated_at timestamp default current_timestamp,
  created_at timestamp default current_timestamp
)