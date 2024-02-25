CREATE TABLE versions (
  id SERIAL PRIMARY KEY,
  platform VARCHAR NOT NULL,
  production VARCHAR NOT NULL,
  beta VARCHAR NOT NULL,
  alpha VARCHAR NOT NULL,
  UNIQUE(platform)
);

INSERT INTO versions(platform, production, beta, alpha) VALUES ('ios', '1.0','1.0','1.0');
INSERT INTO versions(platform, production, beta, alpha) VALUES ('android', '1.0','1.0','1.0');