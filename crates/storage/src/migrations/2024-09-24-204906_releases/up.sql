CREATE TABLE releases (
  platform_store platform_store PRIMARY KEY NOT NULL,

  version VARCHAR(32) NOT NULL,
  upgrade_required bool NOT NULL default false,
  update_enabled bool NOT NULL default true,

  updated_at timestamp NOT NULL default current_timestamp,
  created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('releases');
