CREATE TABLE assets_types (
    id VARCHAR(32) PRIMARY KEY NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

INSERT INTO assets_types(id) VALUES ('NATIVE');
INSERT INTO assets_types(id) VALUES ('ERC20');
INSERT INTO assets_types(id) VALUES ('BEP20');
INSERT INTO assets_types(id) VALUES ('BEP2');
INSERT INTO assets_types(id) VALUES ('SPL');
INSERT INTO assets_types(id) VALUES ('TRC20');

SELECT diesel_manage_updated_at('assets_types');