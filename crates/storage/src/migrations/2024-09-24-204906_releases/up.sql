CREATE TABLE releases (
  platform_store VARCHAR(32) PRIMARY KEY  NOT NULL,
  
  version VARCHAR(32) NOT NULL,
  upgrade_required bool NOT NULL default false,
  
  updated_at timestamp NOT NULL default current_timestamp,
  created_at timestamp NOT NULL default current_timestamp
);


SELECT diesel_manage_updated_at('releases');