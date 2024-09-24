CREATE TABLE releases (
  id SERIAL PRIMARY KEY,
  platform_store VARCHAR(32) NOT NULL,
  version VARCHAR(32) NOT NULL,
  upgrade_required bool NOT NULL default false,

  UNIQUE(platform_store)
);

DROP TABLE IF EXISTS versions;
