CREATE TABLE releases (
  id SERIAL PRIMARY KEY,
  platform_store VARCHAR(32) NOT NULL,
  version VARCHAR(32) NOT NULL,
  upgrade_required bool NOT NULL default false,

  UNIQUE(platform_store)
);

INSERT INTO releases(platform_store, version) VALUES ('appstore', '1.0');
INSERT INTO releases(platform_store, version) VALUES ('playstore', '1.0');

DROP TABLE IF EXISTS versions;
